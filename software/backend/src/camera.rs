use anyhow::Result;
use bytes::Bytes;
use gstreamer::prelude::GObjectExtManualGst;
use gstreamer::prelude::ToSendValue;
use gstreamer::{self as gst, glib::object::ObjectExt};
use gstreamer_app as gst_app;
use gstreamer_video::{VideoFormat, VideoFrameRef, VideoInfo};
use std::sync::Arc;
use tokio::sync::watch;
use turbojpeg::{Compressor, Subsamp, YuvImage};

use communication::ws as com_ws;

pub const STREAM_WIDTH: u32 = 1440;
pub const STREAM_HEIGHT: u32 = 810;

pub const PHOTO_WIDTH: u32 = 4056;
pub const PHOTO_HEIGHT: u32 = 3040;

pub trait CameraPropertiesExt {
    fn write_to_source(&self, source: &gst::Element);
    fn patch(&mut self, other: &Self) -> usize;
}

impl CameraPropertiesExt for com_ws::parameters::CameraProperties {
    fn write_to_source(&self, source: &gst::Element) {
        if let Some(v) = self.exposure_time
            && source.has_property("exposure-time")
        {
            source.set_property("exposure-time", v as i32);
        }
        if let Some(v) = self.brightness
            && source.has_property("brightness")
        {
            source.set_property("brightness", v);
        }
        if let Some(v) = self.contrast
            && source.has_property("contrast")
        {
            source.set_property("contrast", v);
        }
        if let Some(v) = self.saturation
            && source.has_property("saturation")
        {
            source.set_property("saturation", v);
        }
        if let Some(v) = self.auto_white_balance
            && source.has_property("awb-enable")
        {
            source.set_property("awb-enable", v);
        }
        if let Some(v) = &self.white_balance_mode
            && source.has_property("awb-mode")
        {
            source.set_property_from_str("awb-mode", &v.to_string());
        }
        if let (Some(r), Some(b)) = (self.color_gain_red, self.color_gain_blue)
            && source.has_property("colour-gains")
        {
            let gains = gst::Array::from_iter([r.to_send_value(), b.to_send_value()]);
            source.set_property("colour-gains", gains);
        }
        if let Some(v) = &self.test_pattern
            && source.has_property("pattern")
        {
            source.set_property_from_str("pattern", &v.to_string());
        }
    }

    fn patch(&mut self, other: &Self) -> usize {
        let mut changes = 0;

        if let Some(v) = other.exposure_time
            && self.exposure_time != other.exposure_time
        {
            self.exposure_time = Some(v);
            changes += 1;
        }
        if let Some(v) = other.gain
            && self.gain != other.gain
        {
            self.gain = Some(v);
            changes += 1;
        }
        if let Some(v) = other.brightness
            && self.brightness != other.brightness
        {
            self.brightness = Some(v);
            changes += 1;
        }
        if let Some(v) = other.contrast
            && self.contrast != other.contrast
        {
            self.contrast = Some(v);
            changes += 1;
        }
        if let Some(v) = other.saturation
            && self.saturation != other.saturation
        {
            self.saturation = Some(v);
            changes += 1;
        }
        if let Some(v) = other.sharpness
            && self.sharpness != other.sharpness
        {
            self.sharpness = Some(v);
            changes += 1;
        }
        if let Some(v) = other.auto_white_balance
            && self.auto_white_balance != other.auto_white_balance
        {
            self.auto_white_balance = Some(v);
            changes += 1;
        }
        if let Some(v) = &other.white_balance_mode
            && self.white_balance_mode.as_ref() != Some(v)
        {
            self.white_balance_mode = Some(v.clone());
            changes += 1;
        }
        if let Some(v) = &other.test_pattern {
            self.test_pattern = Some(v.clone());
            changes += 1;
        }
        if let Some(r) = other.color_gain_red
            && self.color_gain_red != other.color_gain_red
        {
            self.color_gain_red = Some(r);
            changes += 1;
        }
        if let Some(b) = other.color_gain_blue
            && self.color_gain_blue != other.color_gain_blue
        {
            self.color_gain_blue = Some(b);
            changes += 1;
        }

        changes
    }
}

pub async fn produce_frames(tx: watch::Sender<Arc<Bytes>>, sink: gst_app::AppSink) {
    let mut tj = Compressor::new().expect("Failed to create TurboJPEG Compressor");
    let info = VideoInfo::builder(VideoFormat::I420, STREAM_WIDTH, STREAM_HEIGHT)
        .build()
        .expect("Failed to create VideoInfo");

    fn take_frame(
        sink: &gst_app::AppSink,
        tj: &mut Compressor,
        info: &VideoInfo,
    ) -> Result<Vec<u8>> {
        let sample = sink.pull_sample()?;
        let buf = sample
            .buffer()
            .ok_or_else(|| anyhow::anyhow!("no buffer"))?;

        let frame = VideoFrameRef::from_buffer_ref_readable(buf, info)?;

        let strides = info.stride();
        let (sy, su, sv) = (
            strides[0] as usize,
            strides[1] as usize,
            strides[2] as usize,
        );

        let (w, h) = (info.width() as usize, info.height() as usize);
        let (cw, ch) = (w / 2, h / 2);

        let y = frame.plane_data(0).unwrap();
        let u = frame.plane_data(1).unwrap();
        let v = frame.plane_data(2).unwrap();

        let mut yuv = Vec::with_capacity(w * h * 3 / 2);

        for row in (0..h).rev() {
            let s = row * sy;
            let slice = &y[s..s + w];
            yuv.extend(slice.iter().rev());
        }

        for row in (0..ch).rev() {
            let s = row * su;
            let slice = &u[s..s + cw];
            yuv.extend(slice.iter().rev());
        }

        for row in (0..ch).rev() {
            let s = row * sv;
            let slice = &v[s..s + cw];
            yuv.extend(slice.iter().rev());
        }

        let yuv_img = YuvImage {
            pixels: yuv.as_slice(),
            width: w,
            align: 1,
            height: h,
            subsamp: Subsamp::Sub2x2,
        };

        tj.set_quality(80)?;
        let jpeg = tj.compress_yuv_to_vec(yuv_img)?; // -> Vec<u8>
        Ok(jpeg)
    }

    loop {
        if let Ok(frame) = take_frame(&sink, &mut tj, &info) {
            let _ = tx.send(Arc::new(Bytes::from(frame)));
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        } else {
            eprintln!("Failed to take frame from sink, using test image");
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        }
    }
}
