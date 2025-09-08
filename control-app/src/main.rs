use anyhow::{Result, anyhow};
use axum::{Router, routing::get, routing::put};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video::{VideoFormat, VideoFrameRef, VideoInfo};
use std::sync::Arc;
use tokio::sync::watch;
use turbojpeg::{Compressor, Subsamp, YuvImage};

mod handlers;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    pipeline: gst::Pipeline,
}

impl AppState {
    async fn new(frame_rx: watch::Receiver<Arc<Bytes>>) -> Result<AppState> {
        let source_element = if cfg!(target_arch = "aarch64") {
            "libcamerasrc name=source exposure-time-mode=manual exposure-time=4000"
        } else {
            "videotestsrc name=source is-live=true pattern=smpte"
        };

        let pipeline_string = format!(
            "{source_element}
            ! video/x-raw,format=I420,width={WIDTH},height={HEIGHT},framerate=20/1
            ! queue max-size-buffers=1 leaky=downstream
            ! appsink name=sink emit-signals=false sync=false drop=true max-buffers=1 enable-last-sample=false"
        );

        gst::init()?;
        let pipeline = gst::parse::launch(&pipeline_string)?
            .downcast::<gst::Pipeline>()
            .map_err(|e| anyhow!("Launching pipeline failed: {}", e.type_().name()))?;

        if cfg!(target_arch = "aarch64") {
            let cam = pipeline.by_name("source").unwrap();
            cam.set_property("awb-enable", false);
        }

        pipeline.set_state(gst::State::Playing)?;

        Ok(AppState { frame_rx, pipeline })
    }

    fn get_sink(&self) -> gst_app::AppSink {
        self.pipeline
            .by_name("sink")
            .unwrap()
            .downcast::<gst_app::AppSink>()
            .unwrap()
    }

    fn get_source(&self) -> gst::Element {
        self.pipeline.by_name("source").unwrap()
    }
}

async fn produce_frames(tx: watch::Sender<Arc<Bytes>>, sink: gst_app::AppSink) {
    let mut tj = Compressor::new().unwrap();

    let info = VideoInfo::builder(VideoFormat::I420, WIDTH, HEIGHT)
        .build()
        .unwrap();

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

        let strides = info.stride(); // -> &[i32; 4] in recent gstreamer-video
        let sy = strides[0] as usize;
        let su = strides[1] as usize;
        let sv = strides[2] as usize;

        let w = info.width() as usize;
        let h = info.height() as usize;
        let cw = w / 2;
        let ch = h / 2;

        let y = frame.plane_data(0).unwrap();
        let u = frame.plane_data(1).unwrap();
        let v = frame.plane_data(2).unwrap();

        // Pack to a tight I420 buffer (align = 1)
        let mut yuv = Vec::with_capacity(w * h * 3 / 2);

        // Y
        for row in 0..h {
            let s = row * sy;
            yuv.extend_from_slice(&y[s..s + w]);
        }
        // U
        for row in 0..ch {
            let s = row * su;
            yuv.extend_from_slice(&u[s..s + cw]);
        }
        // V
        for row in 0..ch {
            let s = row * sv;
            yuv.extend_from_slice(&v[s..s + cw]);
        }

        // Describe the packed buffer and encode
        let yuv_img = YuvImage {
            pixels: yuv.as_slice(),
            width: w,
            align: 1, // tightly packed rows
            height: h,
            subsamp: Subsamp::Sub2x2, // I420 = 4:2:0
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

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));

    let app_state = AppState::new(rx).await.unwrap();
    let sink = app_state.get_sink();

    tokio::spawn(produce_frames(tx, sink));

    let app = Router::new()
        .route("/stream", get(handlers::get_stream_mjpeg))
        .route(
            "/camera/property/{name}",
            put(handlers::put_camera_property),
        )
        .route(
            "/camera/property/{name}",
            get(handlers::get_camera_property),
        )
        .with_state(app_state);

    let addr = "0.0.0.0:3000";
    println!("listening on http://{addr}/stream.mjpg");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
