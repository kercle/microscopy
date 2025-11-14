use axum::extract::{Path, Query, State};
use axum::{body::Body, response::IntoResponse};
use bytes::Bytes;
use interface::uart::{HostCommand, StageMotorCmd};
use http::Response;
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::control_app::AppState;

#[macro_use]
mod macros;

pub mod z_scan;

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

pub async fn cancel_operation(State(state): State<AppState>) -> impl IntoResponse {
    warn!("Operation was cancelled by user request");

    state.cancel_operation().await;
    json_response!(200, json_string!("ok"))
}

pub async fn take_photo(State(state): State<AppState>) -> impl IntoResponse {
    info!("Received take photo request");

    let parameters = {
        let p = state.parameters_controller.read().await;
        p.parameters.clone()
    };

    let app_state_guard = match state.with_guard().await {
        Ok(guard) => guard,
        Err(err) => {
            error!("Failed to acquire app state guard: {err}");
            return json_response!(
                500,
                json_string!(&format!("Failed to acquire app state guard: {err}"))
            );
        }
    };

    match app_state_guard.take_photo(&parameters).await {
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

#[derive(Deserialize)]
pub struct CommandQuery {
    steps: Option<i32>,
    step_delay_us: Option<u32>,
    limit: Option<i32>,
}

pub async fn stage_z_motor_command(
    State(_state): State<AppState>,
    Path(path): Path<String>,
    query: Query<CommandQuery>,
) -> impl IntoResponse {
    if let Some(driver) = &_state.device_driver {
        let mut driver = driver.lock().await;
        let cmd_query: CommandQuery = query.0;

        let command = match path.as_str() {
            "steps" => StageMotorCmd::MoveSteps {
                steps: cmd_query.steps.unwrap_or(100),
                step_delay_us: cmd_query.step_delay_us.unwrap_or(1000),
            },
            "home" => StageMotorCmd::Home,
            "enable" => StageMotorCmd::Enable,
            "disable" => StageMotorCmd::Disable,
            "stop" => StageMotorCmd::Stop,
            "set_lower_limit" => {
                if let Some(limit) = cmd_query.limit {
                    StageMotorCmd::SetLowerLimit(limit)
                } else {
                    StageMotorCmd::SetLowerLimitToCurrent
                }
            }
            "set_upper_limit" => {
                if let Some(limit) = cmd_query.limit {
                    StageMotorCmd::SetUpperLimit(limit)
                } else {
                    StageMotorCmd::SetUpperLimitToCurrent
                }
            }
            "release_limits" => StageMotorCmd::ReleaseLimits,
            "goto_lower_limit" => StageMotorCmd::GoToLowerLimit {
                step_delay_us: 1000,
            },
            "goto_upper_limit" => StageMotorCmd::GoToUpperLimit {
                step_delay_us: 1000,
            },
            _ => {
                return json_response!(
                    400,
                    json_string!("Invalid command, use 'up', 'down', or 'stop'")
                );
            }
        };

        match driver
            .send_command::<String>(HostCommand::StageMotor(command))
            .await
        {
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
