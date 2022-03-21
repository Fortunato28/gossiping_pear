use std::time::Duration;

use futures::StreamExt;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::nothing_to_do;

async fn gossiping(tx: UnboundedSender<Message>, period: u32) {
    loop {
        // TODO make random message
        match tx.unbounded_send(Message::Text("test".to_string())) {
            Ok(_) => nothing_to_do(),
            Err(_) => {
                dbg!(&"Unable to send message!");
                return;
            }
        }

        dbg!(&"Still working");

        tokio::time::sleep(Duration::from_secs(period.into())).await;
    }
}

// TODO several connections
async fn client_behavior(period: u32, connection: Vec<String>) {
    let (ws_stream, _) = connect_async(format!("ws://{}", connection[0]))
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (outgoing, _) = ws_stream.split();
    let (tx, rx) = unbounded();

    tokio::spawn(gossiping(tx, period));
    let _ = rx.map(Ok).forward(outgoing).await;
}

pub fn run_client(period: u32, connection: Vec<String>) {
    tokio::spawn(client_behavior(period, connection));
}
