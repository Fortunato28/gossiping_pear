use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{gossiping::gossiping, nothing_to_do};

async fn client_behavior(period: u32, connection: String) {
    let (ws_stream, _) = connect_async(format!("ws://{}", connection))
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (outgoing, incoming) = ws_stream.split();
    let (tx, rx) = unbounded();

    // TODO probably extract shared incoming behavior with server part
    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            connection,
            msg.to_text().unwrap()
        );

        future::ok(())
    });

    tokio::spawn(gossiping(tx, period));
    let sending_process = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, sending_process);
    future::select(broadcast_incoming, sending_process).await;
}

pub fn run_client(period: u32, connection: Vec<String>) {
    connection.iter().for_each(|connection| {
        tokio::spawn(client_behavior(period, connection.clone()));
    })
}
