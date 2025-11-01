use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path};
use axum::{Router, routing::get, routing::patch, routing::post};
use bytes::Bytes;
use tokio::sync::watch;
use tracing::{error, info, warn};
use clap::Parser;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use communication::driver::DeviceDriver;

use crate::parameters::ParametersController;
use crate::control_app::AppState;
use crate::device_monitor;
use crate::handlers;
use crate::camera;

#[derive(Parser)]
pub struct ServeOptions {
    #[clap(long, default_value = "/tmp/microscope_zscans")]
    z_scan_dir: PathBuf,
}

impl ServeOptions {
    async fn get_z_scan_dir(&self) -> PathBuf {
        tokio::fs::create_dir_all(&self.z_scan_dir)
            .await
            .expect("Failed to create z-scan directory");
        self.z_scan_dir.clone()
    }
}

#[derive(Parser)]
pub enum CliCommand {
    Serve(ServeOptions),
}

fn init_tracing(app_state: &AppState) {
    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .with(app_state.clone())
        .init();
    tracing_gstreamer::integrate_events();
    gstreamer::log::remove_default_log_function();
}

pub async fn serve(options: ServeOptions) -> AppState {
    let parameters_controller = ParametersController::new();
    let mut state_notify = parameters_controller.subscribe_changes();

    let (frame_tx, frame_rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));
    let logs = Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let device_driver =
        if let Ok(device_driver) = DeviceDriver::new(&PathBuf::from("/dev/ttyUSB0"), 115200) {
            info!("Connecting to device on /dev/ttyUSB0");
            Some(Arc::new(tokio::sync::Mutex::new(device_driver)))
        } else {
            warn!("No device found on /dev/ttyUSB0");
            None
        };
    let app_state =
        AppState::new(frame_rx, logs, device_driver, parameters_controller)
            .await
            .unwrap();

    init_tracing(&app_state);
    info!("Starting control-app backend");

    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        if let Err(err) = device_monitor(app_state_clone.clone()).await {
            error!("Device monitor error: {err}");
        }
    });

    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        let app_state = app_state_clone;

        loop {
            tokio::select! {
                ret = state_notify.changed() => {
                    if ret.is_err() {
                        // Sender dropped
                        break;
                    }

                    let params = {
                        let p = state_notify.borrow_and_update();
                        p.clone()
                    };

                    let app_state_guard = app_state.with_guard().await.unwrap();
                    app_state_guard.stop_pipeline().unwrap();
                    params.camera_properties.write_to_source(&app_state.get_source());
                    app_state_guard.play_pipeline().unwrap();
                }
            }
        }
    });

    let sink = app_state.get_sink();
    tokio::spawn(camera::produce_frames(frame_tx, sink));

    let api_routes = Router::new()
        .route("/stream", get(handlers::stream::get_stream_mjpeg))
        .route("/ws", get(handlers::ws::ws_handler))
        .route("/parameters", patch(handlers::rest::patch_parameters))
        .route("/parameters", get(handlers::rest::get_parameters))
        .route(
            "/update/self",
            post(handlers::rest::update_self).layer(DefaultBodyLimit::max(100 * 1024 * 1024)),
        )
        .route("/update/firmware", post(handlers::rest::update_firmware))
        .route("/cancel_operation", get(handlers::rest::cancel_operation))
        .route("/photo", get(handlers::rest::take_photo))
        .route(
            "/stage_z/{:command}",
            get(handlers::rest::stage_z_motor_command),
        )
        .nest(
            "/z-scan",
            handlers::rest::z_scan::get_router(app_state.clone(), options.get_z_scan_dir().await),
        )
        .with_state(app_state.clone());

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/{*path}", get(handlers::asset_response))
        .route(
            "/",
            get(|| handlers::asset_response(Path("index.html".to_string()))),
        );

    let addr = "0.0.0.0:3000";
    info!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    app_state
}
