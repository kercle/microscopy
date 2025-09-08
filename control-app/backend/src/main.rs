use anyhow::{Result, anyhow};
use axum::extract::Path;
use axum::{Router, routing::get, routing::put};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::sync::Arc;
use tokio::sync::watch;

use camera::CameraProperties;
use camera::{HEIGHT, WIDTH};

mod camera;
mod handlers;

pub struct Parameters {
    version: u64,
    camera_properties: CameraProperties,
}

impl Parameters {
    fn update_camera_properties(&mut self, new_props: CameraProperties) {
        self.camera_properties = new_props;
        self.version += 1;
    }
}

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    pipeline: gst::Pipeline,
    parameters: Arc<Parameters>,
}

impl AppState {
    async fn new(
        frame_rx: watch::Receiver<Arc<Bytes>>,
        parameters: Arc<Parameters>,
    ) -> Result<AppState> {
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

        Ok(AppState {
            frame_rx,
            pipeline,
            parameters,
        })
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

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));
    let parameters = Arc::new(Parameters {
        version: 1,
        camera_properties: CameraProperties {
            exposure_time: Some(4000),
            gain: Some(1.0),
            brightness: Some(0),
            contrast: Some(0),
            saturation: Some(0),
            sharpness: Some(0),
            awb_enable: Some(true),
            test_pattern: None,
        },
    });

    let app_state = AppState::new(rx, parameters).await.unwrap();
    let sink = app_state.get_sink();

    tokio::spawn(camera::produce_frames(tx, sink));

    let api_routes = Router::new()
        .route("/stream", get(handlers::stream::get_stream_mjpeg))
        .route("/ws", get(handlers::ws::ws_handler))
        .route(
            "/camera/properties",
            put(handlers::rest::put_camera_properties),
        )
        .route(
            "/camera/properties",
            get(handlers::rest::get_camera_properties),
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
