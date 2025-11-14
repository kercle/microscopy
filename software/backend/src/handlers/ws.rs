use anyhow::Result;
use axum::response::IntoResponse;
use axum::{
    extract::State,
    extract::ws::{self, WebSocket, WebSocketUpgrade},
};
use serde_json::json;
use tracing::{error, info};

use crate::control_app::AppState;
use interface::ws as ws_com;

enum PeerRole {
    Unregistered,
    UserClient,
    ComputeNode,
}

struct WsConnection {
    role: PeerRole,
    app: AppState,
    socket: WebSocket,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    info!("New WebSocket connection");

    ws.on_upgrade(async move |socket| {
        if let Err(err) = handle_socket(socket, app).await {
            eprintln!("Error handling WebSocket: {}", err);
        }
    })
}

async fn process_text_message(text: &str, conn: &mut WsConnection) {
    let msg = if let Ok(v) = serde_json::from_str::<ws_com::WebSocketMessage>(&text) {
        v
    } else {
        error!("Failed to parse JSON from text message");
        return;
    };

    match msg {
        ws_com::WebSocketMessage::UpdateParameters(new_params) => {
            let mut p = conn.app.parameters_controller.write().await;
            p.patch(&new_params);
        }
        ws_com::WebSocketMessage::Logs(_) => {
            // logs from client are ignored
        }
        _ => {
            error!("Unsupported WebSocket message type received");
        }
    }
}

async fn process_message(msg: ws::Message, conn: &mut WsConnection) {
    match msg {
        ws::Message::Text(text) => {
            process_text_message(&text, conn).await;
        }
        ws::Message::Binary(_bin) => {
            // Handle binary message
        }
        ws::Message::Close(_close_frame) => {
            // Handle close message
        }
        ws::Message::Ping(_ping) => {
            // Handle ping
        }
        ws::Message::Pong(_pong) => {
            // Handle pong
        }
    }
}

async fn handle_socket(socket: WebSocket, app: AppState) -> Result<()> {
    let mut conn = WsConnection {
        role: PeerRole::Unregistered,
        app: app.clone(),
        socket,
    };

    let mut rx = {
        let p = conn.app.parameters_controller.read().await;
        p.subscribe_changes()
    };

    let mut log_rx = conn.app.subscribe_to_logs().await;

    let payload_json = {
        let params_guard = conn.app.parameters_controller.read().await;
        let payload = ws_com::WebSocketMessage::UpdateParameters(params_guard.parameters.clone());
        serde_json::to_string(&payload).unwrap()
    };

    conn.socket
        .send(ws::Message::Text(payload_json.into()))
        .await?;

    let msg = serde_json::to_string(&json!({
        "logs": conn.app.get_logs().await
    }));

    conn.socket
        .send(ws::Message::Text(msg.unwrap().into()))
        .await?;

    loop {
        tokio::select! {
            msg = conn.socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        process_message(msg, &mut conn).await;
                    }
                    Some(Err(err)) => {
                        eprintln!("WebSocket error: {}", err);
                        break;
                    }
                    None => break, // Connection closed
                }
            }
            Ok(_) = rx.changed() => {
                let params = rx.borrow_and_update().clone();

                let payload = ws_com::WebSocketMessage::UpdateParameters(params);
                let payload_json = serde_json::to_string(&payload).unwrap();
                conn.socket.send(ws::Message::Text(payload_json.into())).await?;
            }
            Ok(log_msg) = log_rx.recv() => {
                let msg = serde_json::to_string(&json!({
                    "logs": [log_msg]
                }));

                conn.socket.send(ws::Message::Text(msg.unwrap().into())).await?;
            }
        }
    }

    Ok(())
}
