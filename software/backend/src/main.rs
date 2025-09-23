use anyhow::Result;
use axum::extract::{DefaultBodyLimit, Path};
use axum::{Router, routing::get, routing::patch, routing::post, routing::put};
use bytes::Bytes;
use communication::DeviceEvent;
use communication::driver::DeviceDriver;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use parameters::ParametersController;

mod camera;
mod control_app;
mod handlers;
mod parameters;

fn init_tracing(app_state: &control_app::AppState) {
    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .with(app_state.clone())
        .init();
    tracing_gstreamer::integrate_events();
    gstreamer::log::remove_default_log_function();
}

async fn device_monitor(app_state: control_app::AppState) -> Result<()> {
    let driver = if let Some(driver) = &app_state.device_driver {
        driver.clone()
    } else {
        warn!("No device driver available, skipping device monitor.");
        return Ok(());
    };

    driver.lock().await.reset()?;

    while !driver.lock().await.connection_established::<String>() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    info!("Connection to device established.");

    loop {
        if let Some(event) = driver.lock().await.recv_event::<String>()? {
            match event {
                DeviceEvent::LogMessage { level, message } => match level {
                    communication::LogMessageLevel::Info => {
                        info!("Controller board log: {message}");
                    }
                    communication::LogMessageLevel::Warning => {
                        warn!("Controller board log: {message}");
                    }
                    communication::LogMessageLevel::Error => {
                        error!("Controller board log: {message}");
                    }
                },
                _ => {}
            }
        } else {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
}

#[tokio::main]
async fn main() {
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
        control_app::AppState::new(frame_rx, logs, device_driver, parameters_controller)
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

                    let params = state_notify.borrow_and_update();

                    app_state.stop_pipeline().unwrap();
                    params.camera_properties.write_to_source(&app_state.get_source());
                    app_state.play_pipeline().unwrap();
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
        .route("/photo", get(handlers::rest::take_photo))
        .route("/stage_z/{:command}", put(handlers::rest::stage_z_motor_command))
        .with_state(app_state);

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
}
