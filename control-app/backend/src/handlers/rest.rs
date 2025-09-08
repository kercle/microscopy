use axum::extract::State;
use axum::{body::Body, response::IntoResponse};
use http::Response;

use crate::AppState;

pub async fn patch_parameters(
    State(_state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .body(Body::from(serde_json::to_string("ok").unwrap()))
        .unwrap()
}

pub async fn get_parameters(
    State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
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
