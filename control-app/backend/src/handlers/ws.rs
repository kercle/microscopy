use axum::response::IntoResponse;
use axum::{
    extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade},
};

use crate::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app))
}

async fn handle_socket(mut socket: WebSocket, app: AppState) {
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        println!("Received a message: {:?}", msg);
                    }
                    Some(Err(err)) => {
                        eprintln!("WebSocket error: {}", err);
                        break;
                    }
                    None => break, // Connection closed
                }
            }
        }
    }
}
