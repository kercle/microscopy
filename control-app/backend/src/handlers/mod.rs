use axum::extract::Path;
use axum::{body::Body, response::IntoResponse, response::Response};
use http::HeaderValue;
use http::StatusCode;
use rust_embed::RustEmbed;

pub mod rest;
pub mod stream;
pub mod ws;

#[derive(RustEmbed)]
#[folder = "../frontend/build"]
struct Assets;

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

    let mut resp = Response::new(Body::from(bytes::Bytes::from(candidate.data.into_owned())));
    let headers = resp.headers_mut();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    headers.insert(http::header::CACHE_CONTROL, HeaderValue::from_static(cache));
    resp
}
