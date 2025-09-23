use axum::extract::{Path, State};
use axum::{body::Body, response::IntoResponse};
use bytes::Bytes;
use communication::{HostCommand, StageMotorCmd};
use http::Response;
use tracing::{error, info};

use crate::control_app::AppState;

macro_rules! json_response {
    ($status:expr, $body:expr) => {
        Response::builder()
            .status($status)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Content-Type", "application/json")
            .body(Body::from($body))
            .unwrap()
    };
}

macro_rules! json_string {
    ($s:expr) => {
        serde_json::to_string($s).unwrap()
    };
}

pub async fn patch_parameters(State(_state): State<AppState>) -> impl IntoResponse {
    json_response!(500, json_string!("not implemented"))
}

pub async fn get_parameters(State(state): State<AppState>) -> impl IntoResponse {
    let body = {
        let p = state.parameters_controller.read().await;
        serde_json::to_string(&p.parameters)
    };

    match body {
        Ok(body) => json_response!(200, body),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to serialize parameters: {err}"))
        ),
    }
}

pub async fn update_self(State(state): State<AppState>, body: Bytes) -> impl IntoResponse {
    info!("Received self-update request, size: {} bytes", body.len());

    match state.update_from_bytes(body).await {
        Ok(_) => json_response!(200, "ok"),
        Err(err) => json_response!(500, json_string!(&format!("Failed to update self: {err}"))),
    }
}

pub async fn update_firmware(State(_state): State<AppState>, _body: Bytes) -> impl IntoResponse {
    json_response!(500, json_string!("not implemented"))
}

pub async fn take_photo(State(state): State<AppState>) -> impl IntoResponse {
    info!("Received take photo request");

    let parameters = {
        let p = state.parameters_controller.read().await;
        p.parameters.clone()
    };

    match state.take_photo(&parameters).await {
        Ok(photo) => {
            let filename = format!(
                "microscope-{}.jpg",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            );
            Response::builder()
                .status(200)
                .header("Cache-Control", "no-cache")
                .header("Pragma", "no-cache")
                .header("Content-Type", "image/jpeg")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{filename}\""),
                )
                .header("Content-Length", photo.len().to_string())
                .body(Body::from(photo))
                .unwrap()
        }
        Err(err) => json_response!(500, json_string!(&format!("Failed to take photo: {err}"))),
    }
}

pub async fn stage_z_motor_command(State(_state): State<AppState>, Path(path): Path<String>) -> impl IntoResponse {
    if let Some(driver) = &_state.device_driver {
        let mut driver = driver.lock().await;

        let command = match path.as_str() {
            "up" => StageMotorCmd::MoveSteps { steps: 100, step_delay_us: 1000 },
            "down" => StageMotorCmd::MoveSteps { steps: -100, step_delay_us: 1000 },
            "stop" => StageMotorCmd::Stop,
            "set_lower_limit" => StageMotorCmd::SetLowerLimit,
            "set_upper_limit" => StageMotorCmd::SetUpperLimit,
            "release_limits" => StageMotorCmd::ReleaseLimits,
            "goto_lower_limit" => StageMotorCmd::GoToLowerLimit { step_delay_us: 1000 },
            "goto_upper_limit" => StageMotorCmd::GoToUpperLimit { step_delay_us: 1000 },
            _ => {
                return json_response!(400, json_string!("Invalid command, use 'up', 'down', or 'stop'"));
            }
        };

        match driver.send_command(HostCommand::StageMotor(command)) {
            Ok(_) => {
                info!("Stage Z motor command sent");
                json_response!(200, json_string!("ok"))
            }
            Err(err) => {
                error!("Failed to send stage Z motor command: {err}");
                json_response!(
                    500,
                    json_string!(&format!("Failed to send stage Z motor command: {err}"))
                )
            }
        }
    } else {
        json_response!(500, json_string!("Device driver not available"))
    }
}
