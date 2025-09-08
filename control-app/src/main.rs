use anyhow::{Result, anyhow};
use axum::{Router, routing::get, routing::put};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::sync::Arc;
use tokio::sync::watch;

mod handlers;

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    pipeline: gst::Pipeline,
}

impl AppState {
    async fn new(frame_rx: watch::Receiver<Arc<Bytes>>) -> Result<AppState> {
        const PIPELINE: &str = if cfg!(target_arch = "aarch64") {
            "libcamerasrc exposure-time=4000 analogue-gain=1.5 awb-mode=daylight
            ! video/x-raw,format=I420,width=1280,height=720,framerate=20/1
            ! queue max-size-buffers=1 leaky=downstream
            ! v4l2convert output-io-mode=mmap capture-io-mode=mmap
            ! videoflip method=rotate-180
            ! v4l2jpegenc output-io-mode=mmap capture-io-mode=mmap qos=false
            ! appsink name=sink emit-signals=false max-buffers=1 drop=true sync=false caps=image/jpeg"
        } else {
            "videotestsrc name=source is-live=true pattern=ball
            ! video/x-raw,format=I420,width=640,height=480,framerate=60/1
            ! jpegenc quality=75
            ! appsink name=sink emit-signals=false max-buffers=1 drop=true sync=false caps=image/jpeg"
        };

        gst::init()?;
        let pipeline = gst::parse::launch(PIPELINE)?
            .downcast::<gst::Pipeline>()
            .map_err(|e| anyhow!("Launching pipeline failed: {}", e.type_().name()))?;

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
    fn take_frame(sink: &gst_app::AppSink) -> Result<Vec<u8>> {
        let sample = sink.pull_sample()?;
        let buf = sample
            .buffer()
            .ok_or_else(|| anyhow::anyhow!("no buffer"))?;
        let map = buf.map_readable().unwrap();
        Ok(map.as_slice().to_vec())
    }

    loop {
        if let Ok(frame) = take_frame(&sink) {
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
