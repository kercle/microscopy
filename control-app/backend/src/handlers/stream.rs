use axum::{body::Body, response::IntoResponse};
use http::{HeaderValue, Response};

use crate::control_app::AppState;

pub async fn get_stream_mjpeg(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let boundary = "microscope-video-frame";
    let content_type = format!("multipart/x-mixed-replace; boundary={boundary}");

    let mut rx = state.subscribe_to_frames();
    let (body_tx, body_rx) =
        tokio::sync::mpsc::channel::<Result<axum::body::Bytes, anyhow::Error>>(5);

    tokio::spawn(async move {
        while let Ok(()) = rx.changed().await {
            let frame = rx.borrow().clone();
            let mut part = format!(
                "--{boundary}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                frame.len()
            )
            .into_bytes();
            part.extend_from_slice(&frame);
            part.extend_from_slice(b"\r\n");

            if body_tx
                .send(Ok(axum::body::Bytes::from(part)))
                .await
                .is_err()
            {
                break; // client disconnected
            }
        }
    });

    let body = Body::from_stream(tokio_stream::wrappers::ReceiverStream::new(body_rx));

    Response::builder()
        .status(200)
        .header(
            "Content-Type",
            HeaderValue::from_str(&content_type).unwrap(),
        )
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .body(body)
        .unwrap()
}
