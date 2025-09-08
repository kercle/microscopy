use anyhow::{Result, anyhow};
use axum::extract::Path;
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

const WIDTH: u32 = 1440;
const HEIGHT: u32 = 810;

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
            cam.set_property("awb-enable", true);
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

        for row in 0..h {
            let s = row * sy;
            yuv.extend_from_slice(&y[s..s + w]);
        }

        for row in 0..ch {
            let s = row * su;
            yuv.extend_from_slice(&u[s..s + cw]);
        }

        for row in 0..ch {
            let s = row * sv;
            yuv.extend_from_slice(&v[s..s + cw]);
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

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));

    let app_state = AppState::new(rx).await.unwrap();
    let sink = app_state.get_sink();

    tokio::spawn(produce_frames(tx, sink));

    let api_routes = Router::new()
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

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/{*path}", get(handlers::asset_response))
        .route(
            "/",
            get(|| handlers::asset_response(Path("index.html".to_string()))),
        );

    let addr = "0.0.0.0:3000";
    println!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
