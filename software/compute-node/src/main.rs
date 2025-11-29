mod focus_stacking;
mod procedure;

use std::{collections::HashMap, env};

use common::ws::{WebSocketMessage, compute_node::ComputeNodeCapabilities, value::Value};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::sync::CancellationToken;

use crate::procedure::Procedure;

#[derive(Clone)]
struct AppState {
    cancel_token: CancellationToken,
    exec_queue_tx: mpsc::Sender<(Box<dyn procedure::Procedure + Send>, HashMap<String, Value>)>,
    focus_stacking: focus_stacking::FocusStacking,
}

async fn react_to_focus_stacking_request(
    sender_tx: &mpsc::UnboundedSender<Message>,
    app_state: &AppState,
    source_uuid: String,
    params: HashMap<String, common::ws::value::Value>,
) {
    let desc = app_state.focus_stacking.describe(params).await;

    let payload = serde_json::to_string(&WebSocketMessage::ProcedureDescription {
        procedure_name: "focus_stacking".to_string(),
        source_uuid: None,
        destination_uuid: source_uuid,
        procedure: desc,
    });

    if let Ok(payload) = payload {
        let _ = sender_tx.send(Message::Text(payload.into()));
    }
}

async fn process_websocket_message(
    msg: WebSocketMessage,
    app_state: &AppState,
    sender_tx: &mpsc::UnboundedSender<Message>,
) {
    match msg {
        WebSocketMessage::WithProcedureParams {
            procedure_name,
            source_uuid,
            destination_uuid: _,
            params,
            ..
        } => {
            if source_uuid.is_none() {
                return;
            }

            if procedure_name == "focus_stacking" && source_uuid.is_some() {
                react_to_focus_stacking_request(sender_tx, app_state, source_uuid.unwrap(), params)
                    .await;
            }
        }
        WebSocketMessage::StartProcedure {
            compute_node_uuid,
            procedure_name,
            params,
        } => {
            if procedure_name == "focus_stacking" {
                let procedure: Box<dyn procedure::Procedure + Send> =
                    Box::new(app_state.focus_stacking.clone());

                if let Err(e) = app_state.exec_queue_tx.try_send((procedure, params)) {
                    println!(
                        "Failed to enqueue FocusStacking procedure for compute node: {compute_node_uuid}, error: {e}"
                    );
                } else {
                    println!(
                        "Enqueued FocusStacking procedure for compute node: {compute_node_uuid}"
                    );
                }
            } else {
                println!("Received StartProcedure for unknown procedure: {procedure_name}");
            }
        }
        _ => {
            // Ignore other message types for now
        }
    }
}

async fn process_message(
    msg: Message,
    app_state: &AppState,
    sender_tx: &mpsc::UnboundedSender<Message>,
) {
    match msg {
        Message::Text(text) => {
            println!("Received text message: {}", text);

            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                process_websocket_message(ws_msg, app_state, sender_tx).await;
            } else {
                println!("Failed to parse WebSocket message");
            }
        }
        Message::Binary(bin) => {
            println!("Received binary message: {:?}", bin);
        }
        Message::Close(frame) => {
            println!("Connection closed: {:?}", frame);
        }
        _ => {
            println!("Received other message: {:?}", msg);
        }
    }
}

async fn processing_thread(
    app_state: AppState,
    mut exec_queue_rx: mpsc::Receiver<(
        Box<dyn procedure::Procedure + Send>,
        HashMap<String, Value>,
    )>,
) {
    loop {
        tokio::select! {
            _ = app_state.cancel_token.cancelled() => {
                break;
            }
            Some((procedure, params)) = exec_queue_rx.recv() => {
                procedure.execute(params).await;
            }
        }
    }
}

async fn receiver_thread(
    app_state: AppState,
    sender_tx: mpsc::UnboundedSender<Message>,
    mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
) {
    loop {
        tokio::select! {
            _ = app_state.cancel_token.cancelled() => {
                break;
            }
            msg = read.next() => {
                println!("Received a message");
                if let Some(Ok(msg)) = msg {
                    process_message(msg, &app_state, &sender_tx).await;
                } else {
                    println!("Connection closed or error occurred");
                    break;
                }
            }
        }
    }

    app_state.cancel_token.cancel();
}

async fn sender_thread(
    app_state: AppState,
    mut sender_rx: mpsc::UnboundedReceiver<Message>,
    mut write: impl SinkExt<Message> + Unpin,
) {
    let capabilities = ComputeNodeCapabilities {
        procedures: HashMap::from([(
            "focus_stacking".to_string(),
            app_state.focus_stacking.describe(HashMap::new()).await,
        )]),
    };

    let payload = serde_json::to_string(&common::ws::WebSocketMessage::RegisterComputeNode(
        capabilities,
    ))
    .unwrap();
    let _ = write.send(Message::Text(payload.into())).await;

    loop {
        tokio::select! {
            _ = app_state.cancel_token.cancelled() => {
                break;
            }
            Some(msg) = sender_rx.recv() => {
                let _ = write.send(msg).await;
            }
        }
    }

    let _ = write.close().await;
}

async fn ctrlc_handler(app_state: AppState) {
    tokio::select! {
        _ = app_state.cancel_token.cancelled() => {
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Ctrl-C received, shutting down...");
            app_state.cancel_token.cancel();
        }
    }
}

#[tokio::main]
async fn main() {
    let host_name = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let (ws_stream, _) = connect_async(format!("ws://{host_name}/api/ws"))
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (exec_queue_tx, exec_queue_rx) =
        mpsc::channel::<(Box<dyn procedure::Procedure + Send>, HashMap<String, Value>)>(100);
    let app_state = AppState {
        cancel_token: CancellationToken::new(),
        exec_queue_tx,
        focus_stacking: focus_stacking::FocusStacking::new(host_name.clone()),
    };

    let (sender_tx, sender_rx) = mpsc::unbounded_channel();
    let (write, read) = ws_stream.split();

    tokio::spawn(ctrlc_handler(app_state.clone()));
    tokio::select! {
        _ = processing_thread(app_state.clone(), exec_queue_rx) => {
        }
        _ = receiver_thread(app_state.clone(), sender_tx, read) => {
        }
        _ = sender_thread(app_state.clone(), sender_rx, write) => {
        }
    }
}
