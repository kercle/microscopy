use std::fmt::Display;

use axum::{
    Json,
    extract::{Path, State},
};
use axum::{body::Body, response::IntoResponse};
use glib::Value as GValue;
use glib::{object::ObjectExt, translate::ToGlibPtr};
use gstreamer::{glib, prelude::GObjectExtManualGst};
use http::StatusCode;
use http::{HeaderValue, Response};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(RustEmbed)]
#[folder = "../frontend/build"]
struct Assets;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestPattern {
    Smpte = 0,
    Snow = 1,
    Ball = 18,
}

impl From<i32> for TestPattern {
    fn from(value: i32) -> Self {
        match value {
            0 => TestPattern::Smpte,
            1 => TestPattern::Snow,
            18 => TestPattern::Ball,
            _ => unimplemented!("Unsupported test pattern value {value}"),
        }
    }
}

impl Into<GValue> for TestPattern {
    fn into(self) -> GValue {
        GValue::from(self as i32)
    }
}

impl Display for TestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestPattern::Smpte => write!(f, "smpte"),
            TestPattern::Snow => write!(f, "snow"),
            TestPattern::Ball => write!(f, "ball"),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "property", content = "value", rename_all = "snake_case")]
pub enum CameraProperty {
    TestPattern(TestPattern),
}

impl Into<GValue> for CameraProperty {
    fn into(self) -> GValue {
        match self {
            CameraProperty::TestPattern(pattern) => pattern.into(),
        }
    }
}

impl Display for CameraProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraProperty::TestPattern(pattern) => write!(f, "{pattern}"),
        }
    }
}

pub async fn put_camera_property(
    State(state): axum::extract::State<AppState>,
    Path(name): Path<String>,
    Json(property): Json<CameraProperty>,
) -> impl IntoResponse {
    let source = state.get_source();

    if !source.has_property(name.as_str()) {
        return Response::builder()
            .status(404)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(format!("Property '{name}' not found")))
            .unwrap();
    }

    source.set_property_from_str(name.as_str(), format!("{property}").as_str());

    Response::builder()
        .status(200)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .body(Body::from("Hello, World!"))
        .unwrap()
}

pub async fn get_camera_property(
    State(state): axum::extract::State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let source = state.get_source();

    if !source.has_property(name.as_str()) {
        return Response::builder()
            .status(404)
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .body(Body::from(format!("Property '{name}' not found")))
            .unwrap();
    }

    let gt = source.property_type(name.as_str()).map(|t| t.name());
    let gv = source.property_value(name.as_str());

    let (status, body) = match gt {
        Some("GstVideoTestSrcPattern") => {
            let value = CameraProperty::TestPattern(TestPattern::from(unsafe {
                glib::gobject_ffi::g_value_get_enum(gv.to_glib_none().0)
            }));
            (200, serde_json::to_string(&value).unwrap())
        }
        Some(t) => (
            500,
            format!("Property '{name}' has unsupported type '{t:?}'"),
        ),
        None => (500, format!("Property '{name}' has unknown type")),
    };

    Response::builder()
        .status(status)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

pub async fn get_stream_mjpeg(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let boundary = "microscope-video-frame";
    let content_type = format!("multipart/x-mixed-replace; boundary={boundary}");

    let mut rx = state.frame_rx.clone();
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

pub async fn asset_response(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    let candidate = if let Some(file) = Assets::get(path) {
        file
    } else if let Some(index) = Assets::get("index.html") {
        index
    } else {
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    };

    let mime = if Assets::get(path).is_some() {
        mime_guess::from_path(path).first_or_octet_stream()
    } else {
        mime_guess::mime::TEXT_HTML_UTF_8
    };

    let cache = if mime == mime_guess::mime::TEXT_HTML_UTF_8 {
        "no-cache"
    } else {
        "public, max-age=31536000, immutable"
    };

    let mut resp =
        axum::response::Response::new(Body::from(bytes::Bytes::from(candidate.data.into_owned())));
    let headers = resp.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    headers.insert(http::header::CACHE_CONTROL, HeaderValue::from_static(cache));
    resp
}
