use anyhow::{Result, anyhow};
use axum::extract::Path;
use axum::{Router, routing::get, routing::patch};
use bytes::Bytes;
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, watch};

use camera::CameraProperties;
use camera::{HEIGHT, WIDTH};

mod camera;
mod handlers;

#[derive(Clone, Serialize, Deserialize)]
pub struct Parameters {
    camera_properties: CameraProperties,
}

impl Parameters {
    fn new() -> Self {
        Parameters {
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
        }
    }

    fn patch(&mut self, other: &Self) {
        self.camera_properties.patch(&other.camera_properties);
    }
}

pub struct ParametersController {
    parameters: Parameters,
    notify_channel: watch::Sender<Parameters>,
}

impl ParametersController {
    fn new() -> Self {
        let (notify_channel, _) = watch::channel(Parameters::new());

        ParametersController {
            parameters: Parameters::new(),
            notify_channel,
        }
    }

    pub fn subscribe_changes(&self) -> watch::Receiver<Parameters> {
        self.notify_channel.subscribe()
    }

    pub fn patch(&mut self, other: &Parameters) {
        self.parameters.patch(&other);
        let _ = self.notify_channel.send(self.parameters.clone());
    }
}

#[derive(Clone)]
pub struct AppState {
    frame_rx: watch::Receiver<Arc<Bytes>>,
    pipeline: gst::Pipeline,
    parameters_controller: Arc<RwLock<ParametersController>>,
}

impl AppState {
    async fn new(
        frame_rx: watch::Receiver<Arc<Bytes>>,
        parameters: ParametersController,
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
            parameters_controller: Arc::new(RwLock::new(parameters)),
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
    let parameters_controller = ParametersController::new();
    let mut state_notify = parameters_controller.subscribe_changes();

    let (frame_tx, frame_rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));
    let app_state = AppState::new(frame_rx, parameters_controller)
        .await
        .unwrap();
    let sink = app_state.get_sink();

    let source = app_state.get_source();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                ret = state_notify.changed() => {
                    if ret.is_err() {
                        // Sender dropped
                        break;
                    }

                    let params = state_notify.borrow_and_update();

                    println!("Parameters changed, updating camera properties");
                    params.camera_properties.write_to_source(&source);
                }
            }
        }
    });

    tokio::spawn(camera::produce_frames(frame_tx, sink));

    let api_routes = Router::new()
        .route("/stream", get(handlers::stream::get_stream_mjpeg))
        .route("/ws", get(handlers::ws::ws_handler))
        .route("/parameters", patch(handlers::rest::patch_parameters))
        .route("/parameters", get(handlers::rest::get_parameters))
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
