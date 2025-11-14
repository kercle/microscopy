use anyhow::Result;
use axum::response::IntoResponse;
use axum::{
    extract::State,
    extract::ws::{self, WebSocket, WebSocketUpgrade},
};
use tokio::sync::{broadcast, watch};
use tracing::{error, info};

use crate::control_app::{AppState, AppStateEvent};
use interface::ws::WebSocketMessage;
use interface::ws::parameters::Parameters;

#[derive(PartialEq)]
enum PeerRole {
    Unregistered,
    UserClient,
    ComputeNode(String),
}

struct Receivers {
    app_events: broadcast::Receiver<AppStateEvent>,
    params: watch::Receiver<Parameters>,
}

struct WsConnection {
    role: PeerRole,
    app: AppState,
    socket: WebSocket,

    receivers: Receivers,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    info!("New WebSocket connection");

    ws.on_upgrade(async move |socket| {
        if let Err(err) = handle_socket(socket, app).await {
            eprintln!("Error handling WebSocket: {}", err);
        }
    })
}

async fn process_parsed_message(msg: WebSocketMessage, conn: &mut WsConnection) -> Result<()> {
    match msg {
        WebSocketMessage::RegisterUserClient => {
            conn.role = PeerRole::UserClient;
            info!("WebSocket client registered as UserClient");

            // Send current parameters
            let payload_json = {
                let params_guard = conn.app.parameters_controller.read().await;
                let payload = WebSocketMessage::UpdateParameters(params_guard.parameters.clone());
                serde_json::to_string(&payload).unwrap()
            };
            conn.socket
                .send(ws::Message::Text(payload_json.into()))
                .await?;

            // Send registered compute nodes
            let compute_nodes = conn.app.list_compute_nodes().await;
            let msg =
                serde_json::to_string(&WebSocketMessage::ComputeNodes(compute_nodes))?;
            conn.socket.send(ws::Message::Text(msg.into())).await?;

            // Send existing logs
            let logs = conn.app.get_logs().await;
            let msg = serde_json::to_string(&WebSocketMessage::Logs(logs))?;
            conn.socket.send(ws::Message::Text(msg.into())).await?;
        }
        WebSocketMessage::RegisterComputeNode(capabilities) => {
            let node_id = conn.app.register_compute_node(&capabilities).await;

            conn.role = PeerRole::ComputeNode(node_id.clone());
            info!(
                "WebSocket client registered as ComputeNode with ID {}",
                node_id
            );
        }
        WebSocketMessage::UpdateParameters(new_params) => {
            let mut p = conn.app.parameters_controller.write().await;
            p.patch(&new_params);
        }
        WebSocketMessage::Logs(_) | WebSocketMessage::ComputeNodes(_) => {
            // message to client only; ignore
        }
    }

    Ok(())
}

async fn process_message(msg: ws::Message, conn: &mut WsConnection) -> Result<()> {
    match msg {
        ws::Message::Text(text) => {
            let msg = if let Ok(v) = serde_json::from_str::<WebSocketMessage>(&text) {
                v
            } else {
                error!("Failed to parse JSON from text message");
                return Ok(());
            };

            process_parsed_message(msg, conn).await?;
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

    Ok(())
}

async fn handle_socket(socket: WebSocket, app: AppState) -> Result<()> {
    let mut conn = WsConnection {
        role: PeerRole::Unregistered,
        app: app.clone(),
        socket,
        receivers: Receivers {
            app_events: app.subscribe_to_app_events().await,
            params: {
                let p = app.parameters_controller.read().await;
                p.subscribe_changes()
            },
        },
    };

    loop {
        tokio::select! {
            msg = conn.socket.recv() => {
                match msg {
                    Some(Ok(msg)) => {
                        process_message(msg, &mut conn).await?;
                    }
                    Some(Err(err)) => {
                        eprintln!("WebSocket error: {}", err);
                        break;
                    }
                    None => break, // Connection closed
                }
            }
            Ok(_) = conn.receivers.params.changed(), if conn.role == PeerRole::UserClient => {
                let params = conn.receivers.params.borrow_and_update().clone();

                let payload = WebSocketMessage::UpdateParameters(params);
                let payload_json = serde_json::to_string(&payload).unwrap();
                conn.socket.send(ws::Message::Text(payload_json.into())).await?;
            }
            Ok(event) = conn.receivers.app_events.recv(), if conn.role == PeerRole::UserClient => {
                match event {
                    AppStateEvent::Log(log_msg) => {
                        let payload = WebSocketMessage::Logs(vec![log_msg]);
                        let msg = serde_json::to_string(&payload);
                        conn.socket.send(ws::Message::Text(msg.unwrap().into())).await?;
                    }
                    AppStateEvent::ComputeNoteUpdate(node_list) => {
                        let payload = WebSocketMessage::ComputeNodes(node_list);
                        let msg = serde_json::to_string(&payload);
                        conn.socket.send(ws::Message::Text(msg.unwrap().into())).await?;
                    }
                };
            }
        }
    }

    if let PeerRole::ComputeNode(node_id) = &conn.role {
        conn.app.unregister_compute_node(node_id).await;
        info!("ComputeNode {} disconnected and unregistered", node_id);
    } else {
        info!("WebSocket client disconnected");
    }

    Ok(())
}
