use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use itertools::Itertools;
use tokio_tungstenite::connect_async;
use tungstenite::{Error, Message};

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
    // TODO handle panic when unable to connect because there are no other peer
    let (ws_stream, _) = connect_async(format!("ws://{}", connection))
        .await
        .expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (outgoing, mut incoming) = ws_stream.split();
    let (tx, rx) = unbounded();

    //let peers_to_connect = incoming.next().await.unwrap().unwrap().to_string();
    //dbg!(&peers_to_connect);

    //// TODO probably extract shared incoming behavior with server part
    //let broadcast_incoming = incoming.try_for_each(move |msg| {
    //    let connection = connection.clone();
    //    println!(
    //        "Received a message from {}: {}",
    //        connection,
    //        msg.to_text().unwrap()
    //    );

    //    future::ok(())
    //});

    let broadcast_incoming = receive_data(incoming, connection, port);

    tokio::spawn(gossiping(tx.clone(), period));
    send_server_port(tx, port).await;

    let sending_process = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, sending_process);
    future::select(broadcast_incoming, sending_process).await;
}

async fn receive_data(
    mut incoming: futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    connection: String,
    port: u16,
) -> Result<(), Error> {
    let peers_to_connect = incoming.next().await.unwrap().unwrap().to_string();
    dbg!(&peers_to_connect);
    let peers_to_connect = peers_to_connect
        .split_whitespace()
        .into_iter()
        .map(|address| address.to_string())
        .collect_vec();
    //.for_each(|address| {
    //    dbg!(address);
    //    tokio::spawn(client_behavior(2, address.to_string(), port));
    //});

    //for address in peers_to_connect {
    //    tokio::spawn(client_behavior(2, address.to_string(), port));
    //}

    // TODO probably extract shared incoming behavior with server part
    let broadcast_incoming = incoming.try_for_each(move |msg| {
        println!(
            "Received a message from {}: {}",
            connection,
            msg.to_text().unwrap()
        );

        future::ok(())
    });

    broadcast_incoming.await
}

pub fn run_client(period: u32, connection: Vec<String>, port: u16) {
    connection.iter().for_each(|connection| {
        tokio::spawn(client_behavior(period, connection.clone(), port));
    })
}
