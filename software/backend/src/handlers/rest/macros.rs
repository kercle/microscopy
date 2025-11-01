#[macro_export]
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

#[macro_export]
macro_rules! json_string {
    ($s:expr) => {
        serde_json::to_string($s).unwrap()
    };
}
