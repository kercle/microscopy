use std::fmt::Display;

use glib::Value as GValue;
use gstreamer::glib;
// use glib::{object::ObjectExt, translate::ToGlibPtr};
// use gstreamer::{prelude::GObjectExtManualGst};
use anyhow::Result;
use bytes::Bytes;
use gstreamer_app as gst_app;
use gstreamer_video::{VideoFormat, VideoFrameRef, VideoInfo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;
use turbojpeg::{Compressor, Subsamp, YuvImage};

pub const WIDTH: u32 = 1440;
pub const HEIGHT: u32 = 810;

#[serde_with::skip_serializing_none]
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CameraProperties {
    pub exposure_time: Option<u32>,
    pub gain: Option<f64>,
    pub brightness: Option<i32>,
    pub contrast: Option<i32>,
    pub saturation: Option<i32>,
    pub sharpness: Option<i32>,
    pub awb_enable: Option<bool>,
    pub test_pattern: Option<TestPattern>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestPattern {
    Smpte = 0,
    Snow = 1,
    Ball = 18,
}

impl From<i32> for TestPattern {
    fn from(value: i32) -> Self {
        match value {
            0 => TestPattern::Smpte,
            1 => TestPattern::Snow,
            18 => TestPattern::Ball,
            _ => unimplemented!("Unsupported test pattern value {value}"),
        }
    }
}

impl Into<GValue> for TestPattern {
    fn into(self) -> GValue {
        GValue::from(self as i32)
    }
}

impl Display for TestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestPattern::Smpte => write!(f, "smpte"),
            TestPattern::Snow => write!(f, "snow"),
            TestPattern::Ball => write!(f, "ball"),
        }
    }
}

// Getting camera properties:
//     let source = state.get_source();

//     if !source.has_property(name.as_str()) {
//         return Response::builder()
//             .status(404)
//             .header("Cache-Control", "no-cache")
//             .header("Pragma", "no-cache")
//             .body(Body::from(format!("Property '{name}' not found")))
//             .unwrap();
//     }

//     source.set_property_from_str(name.as_str(), format!("{property}").as_str());

// Setting camera properties:
// let source = state.get_source();

// if !source.has_property(name.as_str()) {
//     return Response::builder()
//         .status(404)
//         .header("Cache-Control", "no-cache")
//         .header("Pragma", "no-cache")
//         .body(Body::from(format!("Property '{name}' not found")))
//         .unwrap();
// }

// let gt = source.property_type(name.as_str()).map(|t| t.name());
// let gv = source.property_value(name.as_str());

// let (status, body) = match gt {
//     Some("GstVideoTestSrcPattern") => {
//         let value = CameraProperty::TestPattern(TestPattern::from(unsafe {
//             glib::gobject_ffi::g_value_get_enum(gv.to_glib_none().0)
//         }));
//         (200, serde_json::to_string(&value).unwrap())
//     }
//     Some(t) => (
//         500,
//         format!("Property '{name}' has unsupported type '{t:?}'"),
//     ),
//     None => (500, format!("Property '{name}' has unknown type")),
// };

pub async fn produce_frames(tx: watch::Sender<Arc<Bytes>>, sink: gst_app::AppSink) {
    let mut tj = Compressor::new().expect("Failed to create TurboJPEG Compressor");
    let info = VideoInfo::builder(VideoFormat::I420, WIDTH, HEIGHT)
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

        let frame = VideoFrameRef::from_buffer_ref_readable(buf, &info)?;

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
