use std::net::SocketAddr;

use anyhow::{anyhow, Context, Result};
use futures::{
    future::{self, BoxFuture},
    pin_mut, FutureExt, StreamExt, TryStreamExt,
};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::connect_async;
use tungstenite::{Error, Message};

use crate::gossiping::gossiping;

async fn send_server_port(tx: UnboundedSender<Message>, port: u16) -> Result<()> {
    tx.unbounded_send(Message::Text(port.to_string()))
        .context("Unable to send port")?;

    Ok(())
}

async fn client_behavior_with_error_propagating(
    period: u32,
    connection: String,
    server_address: SocketAddr,
) -> Result<()> {
    let (ws_stream, _) = connect_async(format!("ws://{}", connection))
        .await
        .context("Failed to connect")?;
    println!("WebSocket handshake has been successfully completed");

    let (outgoing, incoming) = ws_stream.split();
    let (tx, rx) = unbounded();

    let data_reseiving = receive_data(incoming, connection, server_address, period);

    tokio::spawn(gossiping(tx.clone(), period));
    send_server_port(tx, server_address.port()).await?;

    let sending_process = rx.map(Ok).forward(outgoing);

    pin_mut!(data_reseiving, sending_process);
    future::select(data_reseiving, sending_process).await;

    Ok(())
}

async fn client_behavior(period: u32, connection: String, server_address: SocketAddr) {
    match client_behavior_with_error_propagating(period, connection, server_address).await {
        Ok(_) => return,
        Err(error) => {
            println!("Error during connection handling: {}", error);
            return;
        }
    };
}

// We need exactly this return type to avoid recursion in async function
fn receive_data(
    mut incoming: futures::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    connected_peer_address: String,
    server_address: SocketAddr,
    period: u32,
) -> BoxFuture<'static, Result<(), Error>> {
    async move {
        let peers_to_connect = match incoming
            .next()
            .await
            .ok_or_else(|| anyhow!("There is no first message received"))
        {
            Ok(peers_to_connect) => match peers_to_connect {
                Ok(peers_to_connect) => peers_to_connect.to_string(),
                Err(error) => {
                    println!("Got error while receiving peers to connect: {}", error);
                    "".to_string()
                }
            },
            Err(error) => {
                println!("{}", error);
                "".to_string()
            }
        };
        dbg!(&peers_to_connect);
        peers_to_connect
            .split_whitespace()
            .into_iter()
            .filter(|address| *address != server_address.to_string())
            .for_each(|address| {
                tokio::spawn(client_behavior(period, address.to_string(), server_address));
            });

        let broadcast_incoming = incoming.try_for_each(move |message| {
            let message = match message.to_text() {
                Ok(message) => message,
                Err(error) => {
                    println!("Unable to cast received message to string: {}", error);
                    return future::ok(());
                }
            };

            println!(
                "Received a message from {}: {:?}",
                connected_peer_address, message,
            );

            future::ok(())
        });

        broadcast_incoming.await
    }
    .boxed()
}

pub fn run_client(period: u32, connection: Vec<String>, server_address: SocketAddr) {
    connection.iter().for_each(|connection| {
        tokio::spawn(client_behavior(period, connection.clone(), server_address));
    })
}
