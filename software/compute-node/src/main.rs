mod helpers;
mod tasks;

use std::{collections::HashMap, env, sync::Arc};

use common::ws::{WebSocketMessage, compute_node::ComputeNodeCapabilities, value::Value};
use futures::stream::{Map, SelectAll, select_all};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::WatchStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::sync::CancellationToken;

use crate::tasks::Task;
use crate::tasks::focus_stacking::FocusStacking;

type TaskPtr = Arc<dyn Task + Send + Sync>;

#[derive(Clone)]
struct AppState {
    cancel_token: CancellationToken,
    exec_queue_tx: mpsc::Sender<(TaskPtr, HashMap<String, Value>)>,
    tasks: HashMap<&'static str, TaskPtr>,
}

impl AppState {
    fn process_dynamic_progress(
        &self,
    ) -> SelectAll<Map<WatchStream<Option<f32>>, impl FnMut(Option<f32>) -> Option<(String, f32)>>>
    {
        let mut streams = Vec::new();

        for (task_id, task_ptr) in self.tasks.iter() {
            let receiver = task_ptr.get_progress_receiver();
            let watch_stream = WatchStream::new(receiver);
            let owned_task_id = task_id.to_string();

            let mapped_stream = watch_stream.map(move |item| {
                if let Some(progress) = item {
                    return Some((owned_task_id.clone(), progress));
                }
                None
            });

            streams.push(mapped_stream);
        }

        select_all(streams)
    }
}

async fn process_websocket_message(
    msg: WebSocketMessage,
    app_state: &AppState,
    sender_tx: &mpsc::UnboundedSender<Message>,
) {
    match msg {
        WebSocketMessage::WithTaskParams {
            task_name,
            source_uuid,
            destination_uuid: _,
            params,
            ..
        } => {
            if source_uuid.is_none() || source_uuid.is_none() {
                return;
            }

            if let Some(task) = app_state.tasks.get(task_name.as_str()) {
                let payload = serde_json::to_string(&WebSocketMessage::TaskDescription {
                    task_name: task_name.clone(),
                    source_uuid: None,
                    destination_uuid: source_uuid.unwrap(),
                    ui_description: task.describe(task_name.clone(), params).await,
                });

                if let Ok(payload) = payload {
                    let _ = sender_tx.send(Message::Text(payload.into()));
                }
            }
        }
        WebSocketMessage::StartTask {
            compute_node_uuid,
            task_name,
            params,
        } => {
            if let Some(task) = app_state.tasks.get(task_name.as_str()) {
                if let Err(e) = app_state.exec_queue_tx.try_send((task.clone(), params)) {
                    println!(
                        "Failed to enqueue FocusStacking task for compute node: {compute_node_uuid}, error: {e}"
                    );
                } else {
                    println!("Enqueued FocusStacking task for compute node: {compute_node_uuid}");
                }
            } else {
                println!("Received StartTask for unknown task: {task_name}");
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
    mut exec_queue_rx: mpsc::Receiver<(TaskPtr, HashMap<String, Value>)>,
) {
    loop {
        tokio::select! {
            _ = app_state.cancel_token.cancelled() => {
                break;
            }
            Some((task, params)) = exec_queue_rx.recv() => {
                task.execute(params).await;
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
    let mut progress_stream = app_state.process_dynamic_progress();

    let mut tasks = HashMap::new();
    for (task_name, task_ptr) in app_state.tasks.iter() {
        let ui_description = task_ptr
            .describe(task_name.to_string(), HashMap::new())
            .await;
        tasks.insert(task_name.to_string(), ui_description);
    }

    let capabilities = ComputeNodeCapabilities { tasks };

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
            Some(msg) = progress_stream.next() => {
                if msg.is_none() {
                    continue;
                }

                let (task_name, progress) = msg.unwrap();
                let payload = serde_json::to_string(&common::ws::WebSocketMessage::TaskProgressUpdate {
                    compute_node_uuid: None,
                    task_name,
                    progress,
                }).unwrap();
                let _ = write.send(Message::Text(payload.into())).await;
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

    // let task_focus_stacking: TaskPtr =
    //     ;

    let (exec_queue_tx, exec_queue_rx) = mpsc::channel::<(TaskPtr, HashMap<String, Value>)>(100);
    let app_state = AppState {
        cancel_token: CancellationToken::new(),
        exec_queue_tx,
        tasks: HashMap::from([(
            "focus_stacking",
            Arc::new(FocusStacking::new(host_name.clone())) as TaskPtr,
        )]),
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
