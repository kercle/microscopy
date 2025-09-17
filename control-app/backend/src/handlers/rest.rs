use axum::extract::State;
use axum::{body::Body, response::IntoResponse};
use bytes::Bytes;
use http::Response;
use tracing::info;

use crate::control_app::AppState;

pub async fn patch_parameters(State(_state): State<AppState>) -> impl IntoResponse {
    Response::builder()
        .status(500)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .body(Body::from(
            serde_json::to_string("not implemented").unwrap(),
        ))
        .unwrap()
}

pub async fn get_parameters(State(state): State<AppState>) -> impl IntoResponse {
    let body = {
        let p = state.parameters_controller.read().await;
        serde_json::to_string(&p.parameters)
    };

    match body {
        Ok(body) => Response::builder()
            .status(200)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap(),
        Err(err) => Response::builder()
            .status(500)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(format!(
                "Failed to serialize camera properties: {err}"
            )))
            .unwrap(),
    }
}

pub async fn update_self(State(state): State<AppState>, body: Bytes) -> impl IntoResponse {
    info!("Received self-update request, size: {} bytes", body.len());

    match state.update_from_bytes(body).await {
        Ok(_) => Response::builder()
            .status(200)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(serde_json::to_string("ok").unwrap()))
            .unwrap(),
        Err(err) => Response::builder()
            .status(500)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(format!("Failed to update self: {err}")))
            .unwrap(),
    }
}

pub async fn update_firmware(State(_state): State<AppState>, _body: Bytes) -> impl IntoResponse {
    Response::builder()
        .status(500)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .body(Body::from(
            serde_json::to_string("not implemented").unwrap(),
        ))
        .unwrap()
}

pub async fn take_photo(State(state): State<AppState>) -> impl IntoResponse {
    info!("Received take photo request");

    let parameters = {
        let p = state.parameters_controller.read().await;
        p.parameters.clone()
    };

    match state.take_photo(&parameters).await {
        Ok(photo) => Response::builder()
            .status(200)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Content-Type", "image/jpeg")
            .body(Body::from(photo))
            .unwrap(),
        Err(err) => Response::builder()
            .status(500)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(format!("Failed to take photo: {err}")))
            .unwrap(),
    }
}
