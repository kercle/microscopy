use anyhow::Result;
use axum::response::IntoResponse;
use axum::{
    extract::State,
    extract::ws::{WebSocket, WebSocketUpgrade},
};
use serde_json::json;
use tracing::info;

use crate::control_app::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> impl IntoResponse {
    info!("New WebSocket connection");

    ws.on_upgrade(async move |socket| {
        if let Err(err) = handle_socket(socket, app).await {
            eprintln!("Error handling WebSocket: {}", err);
        }
    })
}

async fn process_message(msg: axum::extract::ws::Message, app: &AppState) {
    match msg {
        axum::extract::ws::Message::Text(text) => {
            if let Ok(new_params) = serde_json::from_str::<crate::parameters::Parameters>(&text) {
                let mut p = app.parameters_controller.write().await;
                p.patch(&new_params);
            } else {
                eprintln!("Failed to parse parameters from text message");
            }
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

async fn handle_socket(mut socket: WebSocket, mut app: AppState) -> Result<()> {
    let mut rx = {
        let p = app.parameters_controller.read().await;
        p.subscribe_changes()
    };

    let mut log_rx = app.subscribe_to_logs().await;

    let params_json = {
        let p = app.parameters_controller.read().await;
        serde_json::to_string(&p.parameters).unwrap()
    };

    socket
        .send(axum::extract::ws::Message::Text(params_json.into()))
        .await?;

    let msg = serde_json::to_string(&json!({
        "logs": app.get_logs().await
    }));

    socket
        .send(axum::extract::ws::Message::Text(msg.unwrap().into()))
        .await?;

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
            res = rx.changed() => {
                if res.is_err() {
                    break; // Sender dropped
                }

                let params = rx.borrow_and_update().clone();

                let params_json = serde_json::to_string(&params).unwrap();
                socket.send(axum::extract::ws::Message::Text(params_json.into())).await?;
            }
            Ok(log_msg) = log_rx.recv() => {
                let msg = serde_json::to_string(&json!({
                    "logs": log_msg
                }));

                socket.send(axum::extract::ws::Message::Text(msg.unwrap().into())).await?;
            }
        }
    }

    Ok(())
}
