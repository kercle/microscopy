use std::{collections::HashMap, env};

use futures_util::{SinkExt, StreamExt};
use interface::ws::compute_node::ComputeNodeCapabilities;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_util::sync::CancellationToken;

mod focus_stacking;

async fn process_message(msg: Message) {
    match msg {
        Message::Text(text) => {
            println!("Received text message: {}", text);
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

async fn receiver_thread(
    cancellation_token: CancellationToken,
    mut read: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
) {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                break;
            }
            msg = read.next() => {
                if let Some(Ok(msg)) = msg {
                    process_message(msg).await;
                } else {
                    println!("Connection closed or error occurred");
                    break;
                }
            }
        }
    }

    cancellation_token.cancel();
}

async fn sender_thread(
    cancellation_token: CancellationToken,
    mut write: impl SinkExt<Message> + Unpin,
) {
    let capabilities = ComputeNodeCapabilities {
        procedures: HashMap::from([(
            "focus_stacking".to_string(),
            focus_stacking::FocusStacking::describe(),
        )]),
    };

    let payload = serde_json::to_string(&interface::ws::WebSocketMessage::RegisterComputeNode(
        capabilities,
    ))
    .unwrap();
    let _ = write.send(Message::Text(payload.into())).await;

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {}
        }
    }

    let _ = write.close().await;
}

async fn ctrlc_handler(cancellation_token: CancellationToken) {
    let _ = tokio::signal::ctrl_c().await;
    cancellation_token.cancel();
}

#[tokio::main]
async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let cancellation_token = CancellationToken::new();

    let rcv_thread = tokio::spawn(receiver_thread(cancellation_token.child_token(), read));
    let snd_thread = tokio::spawn(sender_thread(cancellation_token.child_token(), write));

    tokio::spawn(ctrlc_handler(cancellation_token));

    let (res_rcv, res_snd) = tokio::join!(rcv_thread, snd_thread);

    if let Err(e) = res_rcv {
        eprintln!("Receiver thread error: {}", e);
    }

    if let Err(e) = res_snd {
        eprintln!("Sender thread error: {}", e);
    }
}
