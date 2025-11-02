use std::io::Cursor;
use std::path::PathBuf;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::{Router, body::Body, response::IntoResponse, routing};
use bytes::Bytes;
use http::Response;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::control_app::AppState;

pub fn get_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/record/{:relative_start_pos}/{:relative_stop_pos}/{:steps_between_layers}",
            routing::get(record),
        )
        .route("/delete/{uuid}", routing::delete(delete))
        .route("/list", routing::get(list))
        .route(
            "/thumbnail/{:uuid}/{:frame_idx}/{:width}",
            routing::get(thumbnail),
        )
        .route("/frame/{:uuid}/{:frame_idx}", routing::get(frame))
        .with_state(app_state)
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

pub async fn record(
    State(state): State<AppState>,
    Path((relative_start_pos, relative_stop_pos, steps_between_layers)): Path<(i32, i32, u32)>,
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
        state.config.z_scan_dir.clone(),
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

pub async fn list(State(state): State<AppState>) -> impl IntoResponse {
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

    match exec(state.config.z_scan_dir.clone()).await {
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

pub async fn delete(State(state): State<AppState>, Path(uuid): Path<String>) -> impl IntoResponse {
    let scan_dir = state.config.z_scan_dir.join(&uuid);

    match tokio::fs::remove_dir_all(&scan_dir).await {
        Ok(_) => json_response!(200, json_string!("ok")),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to delete z-scan: {err}"))
        ),
    }
}

pub async fn frame(
    State(state): State<AppState>,
    Path((uuid, frame_idx)): Path<(String, usize)>,
) -> impl IntoResponse {
    let frame_path = state
        .config
        .z_scan_dir
        .join(&uuid)
        .join(format!("frame-{:0>4}.jpg", frame_idx));

    match tokio::fs::read(&frame_path).await {
        Ok(frame_data) => Response::builder()
            .status(200)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Content-Type", "image/jpeg")
            .header("Content-Length", frame_data.len().to_string())
            .body(Body::from(frame_data))
            .unwrap(),
        Err(err) => json_response!(
            500,
            json_string!(&format!("Failed to read z-scan frame: {err}"))
        ),
    }
}

pub async fn thumbnail(
    State(state): State<AppState>,
    Path((uuid, frame_idx, bound)): Path<(String, usize, u32)>,
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

    match exec(state.config.z_scan_dir.clone(), uuid, frame_idx, bound).await {
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
