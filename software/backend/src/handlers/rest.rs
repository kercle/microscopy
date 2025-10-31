use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::{body::Body, response::IntoResponse};
use bytes::Bytes;
use communication::{HostCommand, StageMotorCmd};
use http::Response;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

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

#[derive(Serialize, Deserialize)]
pub struct ZScanMetadata {
    pub relative_start_pos: i32,
    pub relative_stop_pos: i32,
    pub steps_between_layers: u32,
    pub frame_count: usize,
    pub uuid: String,
    pub timestamp: chrono::DateTime<chrono::Local>,
}

pub async fn z_scan(
    State(state): State<AppState>,
    Path((relative_start_pos, relative_stop_pos, steps_between_layers)): Path<(i32, i32, u32)>,
    z_scan_dir: PathBuf,
) -> impl IntoResponse {
    async fn exec(
        state: &AppState,
        relative_start_pos: i32,
        relative_stop_pos: i32,
        steps_between_layers: u32,
        z_scan_dir: PathBuf,
        z_scan_uuid: String,
    ) -> Result<String> {
        info!("Received z-scan request");

        let parameters = {
            let p = state.parameters_controller.read().await;
            p.parameters.clone()
        };

        let app_state_guard = state.with_guard().await?;
        let frames = app_state_guard
            .z_scan(
                &parameters,
                relative_start_pos,
                relative_stop_pos,
                steps_between_layers,
            )
            .await?;

        let z_scan_dir = z_scan_dir.join(&z_scan_uuid);
        tokio::fs::create_dir_all(&z_scan_dir).await?;

        for (idx, frame) in frames.iter().enumerate() {
            let filename = format!("{}/frame-{:0>4}.jpg", z_scan_dir.display(), idx);

            tokio::fs::write(&filename, frame).await?;
            info!("Wrote z-scan frame to file {}", filename);
        }

        let metadata = ZScanMetadata {
            relative_start_pos,
            relative_stop_pos,
            steps_between_layers,
            frame_count: frames.len(),
            uuid: z_scan_uuid,
            timestamp: chrono::Local::now(),
        };

        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        let metadata_filename = z_scan_dir.join("metadata.json");
        tokio::fs::write(&metadata_filename, &metadata_json).await?;

        Ok(metadata_json)
    }

    let z_scan_uuid = uuid::Uuid::new_v4().to_string();

    match exec(
        &state,
        relative_start_pos,
        relative_stop_pos,
        steps_between_layers,
        z_scan_dir,
        z_scan_uuid,
    )
    .await
    {
        Ok(metadata) => json_response!(200, metadata),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to perform z-scan: {err}"))
        ),
    }
}

pub async fn list_z_scans(
    State(_state): State<AppState>,
    z_scan_dir: PathBuf,
) -> impl IntoResponse {
    async fn exec(z_scan_dir: PathBuf) -> Result<Vec<ZScanMetadata>> {
        let mut scans = Vec::new();

        let mut dir_entries = tokio::fs::read_dir(&z_scan_dir).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let metadata_path = path.join("metadata.json");

            if !metadata_path.exists() {
                warn!(
                    "Z-scan metadata file does not exist: {}",
                    metadata_path.display()
                );
                continue;
            }

            let metadata_content = tokio::fs::read_to_string(&metadata_path).await?;
            let metadata: ZScanMetadata = serde_json::from_str(&metadata_content)?;

            if metadata.uuid != path.file_name().unwrap().to_string_lossy() {
                warn!(
                    "Z-scan directory name does not match UUID in metadata: {} vs {}, skipping",
                    path.file_name().unwrap().to_string_lossy(),
                    metadata.uuid
                );
                continue;
            }
            scans.push(metadata);
        }

        Ok(scans)
    }

    match exec(z_scan_dir).await {
        Ok(scans) => match serde_json::to_string(&scans) {
            Ok(body) => json_response!(200, body),
            Err(err) => json_response!(
                500,
                json_string!(&format!("Failed to serialize z-scan list: {err}"))
            ),
        },
        Err(err) => json_response!(500, json_string!(&format!("Failed to list z-scans: {err}"))),
    }
}

pub async fn delete_z_scan(
    State(_state): State<AppState>,
    Path(uuid): Path<String>,
    z_scan_dir: PathBuf,
) -> impl IntoResponse {
    let scan_dir = z_scan_dir.join(&uuid);

    match tokio::fs::remove_dir_all(&scan_dir).await {
        Ok(_) => json_response!(200, json_string!("ok")),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to delete z-scan: {err}"))
        ),
    }
}

pub async fn z_scan_thumbnail(
    State(_state): State<AppState>,
    Path((uuid, frame_idx, bound)): Path<(String, usize, u32)>,
    z_scan_dir: PathBuf,
) -> impl IntoResponse {
    async fn exec(z_scan_dir: PathBuf, uuid: String, frame_id: usize, bound: u32) -> Result<Bytes> {
        let thumbnail_dir = z_scan_dir
            .join(&uuid)
            .join("thumbnails")
            .join(format!("{bound}"));
        tokio::fs::create_dir_all(&thumbnail_dir).await?;

        let frame_thumbnail_file = thumbnail_dir.join(format!("frame-{:0>4}.jpg", frame_id));

        if tokio::fs::try_exists(&frame_thumbnail_file).await? {
            let thumbnail_data = tokio::fs::read(&frame_thumbnail_file).await?;
            return Ok(Bytes::from(thumbnail_data));
        }

        let frame_path = z_scan_dir
            .join(&uuid)
            .join(format!("frame-{:0>4}.jpg", frame_id));

        let frame_data = tokio::fs::read(&frame_path).await?;
        let img = image::load_from_memory(&frame_data)?;

        let thumbnail = img.thumbnail(bound, bound);

        let mut thumbnail_data: Vec<u8> = Vec::new();
        let mut cursor: Cursor<&mut Vec<u8>> = Cursor::new(&mut thumbnail_data);
        thumbnail.write_to(&mut cursor, image::ImageFormat::Jpeg)?;

        tokio::fs::write(&frame_thumbnail_file, &thumbnail_data).await?;

        Ok(Bytes::from(thumbnail_data))
    }

    match exec(z_scan_dir, uuid, frame_idx, bound).await {
        Ok(thumbnail_data) => Response::builder()
            .status(200)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Content-Type", "image/jpeg")
            .header("Content-Length", thumbnail_data.len().to_string())
            .body(Body::from(thumbnail_data))
            .unwrap(),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to generate z-scan thumbnail: {err}"))
        ),
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
