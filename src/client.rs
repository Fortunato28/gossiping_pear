use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

use crate::{gossiping::gossiping, nothing_to_do};

async fn send_server_port(tx: UnboundedSender<Message>, port: u16) {
    match tx.unbounded_send(Message::Text(port.to_string())) {
        Ok(_) => nothing_to_do(),
        Err(_) => {
            dbg!(&"Unable to send port!");
        }
    }
}

async fn client_behavior(period: u32, connection: String, port: u16) {
    // TODO handle panic when unable to connect
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

    send_server_port(tx.clone(), port).await;
    tokio::spawn(gossiping(tx, period));
    let sending_process = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, sending_process);
    future::select(broadcast_incoming, sending_process).await;
}

pub fn run_client(period: u32, connection: Vec<String>, port: u16) {
    connection.iter().for_each(|connection| {
        tokio::spawn(client_behavior(period, connection.clone(), port));
    })
}
