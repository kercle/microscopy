use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::extract::{DefaultBodyLimit, Path};
use axum::{Router, routing::get, routing::patch, routing::post};
use bytes::Bytes;
use clap::Parser;
use communication::uart::DeviceEvent;
use tokio::sync::watch;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use communication::uart::driver::DeviceDriver;
use communication::ws as com_ws;

use crate::camera::{self, CameraPropertiesExt};
use crate::control_app::AppState;
use crate::handlers;
use crate::parameters::ParametersController;

#[derive(Parser)]
pub struct Command {
    #[clap(long, default_value = "/tmp/microscope_zscans")]
    z_scan_dir: PathBuf,
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

async fn device_monitor(app_state: AppState) -> Result<()> {
    let driver = if let Some(driver) = &app_state.device_driver {
        driver.clone()
    } else {
        warn!("No device driver available, skipping device monitor.");
        return Ok(());
    };

    {
        let mut driver = driver.lock().await;

        driver.reset().await?; // This also runs the homing procedure
        driver.set_upper_limit::<String>(5450).await?; // dummy value for now
    }

    while !driver.lock().await.connection_established::<String>() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    info!("Connection to device established.");

    loop {
        if let Some(event) = driver.lock().await.recv_event::<String>()? {
            match event {
                DeviceEvent::LogMessage { level, message } => match level {
                    communication::uart::LogMessageLevel::Info => {
                        info!("Controller board log: {message}");
                    }
                    communication::uart::LogMessageLevel::Warning => {
                        warn!("Controller board log: {message}");
                    }
                    communication::uart::LogMessageLevel::Error => {
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

async fn camera_properties_update_loop(
    app_state: AppState,
    mut state_notify: tokio::sync::watch::Receiver<com_ws::parameters::Parameters>,
) {
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
}

impl Command {
    async fn get_z_scan_dir(&self) -> PathBuf {
        tokio::fs::create_dir_all(&self.z_scan_dir)
            .await
            .expect("Failed to create z-scan directory");
        self.z_scan_dir.clone()
    }

    async fn serve_routes(app_state: AppState) {
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
                handlers::rest::z_scan::get_router(app_state.clone()),
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
    }

    pub async fn exec(&self) {
        let parameters_controller = ParametersController::new();
        let state_notify = parameters_controller.subscribe_changes();

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
        let config = crate::control_app::Config {
            z_scan_dir: self.get_z_scan_dir().await,
        };
        let app_state = AppState::new(frame_rx, logs, device_driver, parameters_controller, config)
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

        tokio::spawn(camera_properties_update_loop(
            app_state.clone(),
            state_notify,
        ));

        let sink = app_state.get_sink();
        tokio::spawn(camera::produce_frames(frame_tx, sink));

        Self::serve_routes(app_state.clone()).await;
    }
}
