use axum::{Router, body::Body, response::IntoResponse, routing::get};
use bytes::Bytes;
use http::{HeaderValue, Response};
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Clone)]
struct AppState {
    rx: watch::Receiver<Arc<Bytes>>, // latest JPEG frame
}

#[tokio::main]
async fn main() {
    let (tx, rx) = watch::channel::<Arc<Bytes>>(Arc::new(Bytes::new()));

    tokio::spawn({
        let tx = tx.clone();
        async move {
            let jpeg_frames = [
                Bytes::from_static(include_bytes!("data/test_image_a.jpg")),
                Bytes::from_static(include_bytes!("data/test_image_b.jpg")),
            ];

            let mut counter = 0;

            loop {
                // In a real app, replace with the bytes from your GStreamer appsink
                let _ = tx.send(Arc::new(jpeg_frames[counter % jpeg_frames.len()].clone()));
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                counter += 1;
            }
        }
    });

    let app = Router::new()
        .route("/stream.mjpg", get(stream_mjpeg))
        .with_state(AppState { rx });

    let addr = "0.0.0.0:3000";
    println!("listening on http://{addr}/stream.mjpg");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn stream_mjpeg(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let boundary = "microscope-video-frame";
    let content_type = format!("multipart/x-mixed-replace; boundary={boundary}");

    let mut rx = state.rx.clone();
    let (body_tx, body_rx) =
        tokio::sync::mpsc::channel::<Result<axum::body::Bytes, anyhow::Error>>(100);

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
