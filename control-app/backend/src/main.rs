use axum::extract::{DefaultBodyLimit, Path};
use axum::{Router, routing::get, routing::patch, routing::post};
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

use parameters::ParametersController;

mod camera;
mod control_app;
mod handlers;
mod parameters;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting control-app backend");

    let parameters_controller = ParametersController::new();
    let mut state_notify = parameters_controller.subscribe_changes();

    let (frame_tx, frame_rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));
    let app_state = control_app::AppState::new(frame_rx, parameters_controller)
        .await
        .unwrap();
    let sink = app_state.get_sink();

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

                    println!("Parameters changed, updating camera properties");
                    app_state.stop_pipeline().unwrap();
                    params.camera_properties.write_to_source(&app_state.get_source());
                    app_state.play_pipeline().unwrap();
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
        .route(
            "/update/self",
            post(handlers::rest::update_self).layer(DefaultBodyLimit::max(100 * 1024 * 1024)),
        )
        .route("/update/firmware", post(handlers::rest::update_firmware))
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
