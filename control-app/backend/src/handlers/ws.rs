use axum::response::IntoResponse;
use axum::{
    extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade},
};

use crate::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app))
}

async fn process_message(msg: axum::extract::ws::Message, app: &mut AppState) {
    match msg {
        axum::extract::ws::Message::Text(text) => {
            println!("Received text message: {}", text);

            // Handle text message
        }
        axum::extract::ws::Message::Binary(bin) => {
            println!("Received binary message: {:?}", bin);
            // Handle binary message
        }
        axum::extract::ws::Message::Close(close_frame) => {
            println!("Received close message: {:?}", close_frame);
            // Handle close message
        }
        axum::extract::ws::Message::Ping(ping) => {
            println!("Received ping: {:?}", ping);
            // Handle ping
        }
        axum::extract::ws::Message::Pong(pong) => {
            println!("Received pong: {:?}", pong);
            // Handle pong
        }
    }
}

async fn handle_socket(mut socket: WebSocket, mut app: AppState) {
    

    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        process_message(msg, &mut app).await;
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
