use anyhow::{anyhow, Context, Result};
use parking_lot::Mutex;
use std::net::SocketAddr;

use crate::{gossiping::gossiping, PeerMap};
use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use itertools::Itertools;
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;

fn send_other_peers(tx: UnboundedSender<Message>, current_peer_map: String) -> Result<()> {
    tx.unbounded_send(Message::Text(current_peer_map))
        .context("Error while trying to share peer map")?;

    Ok(())
}

async fn receive_connected_peer_address(
    incoming: &mut futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<TcpStream>>,
    connected_peer_address: SocketAddr,
) -> Result<SocketAddr> {
    let connected_peer_listenting_port = incoming
        .next()
        .await
        .ok_or_else(|| anyhow!("Unable to receive connected peer server port"))??
        .to_string();

    Ok(SocketAddr::new(
        connected_peer_address.ip(),
        connected_peer_listenting_port
            .parse()
            .context("Unable to parse port as u16")?,
    ))
}

async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    connected_peer_address: SocketAddr,
    period: u32,
) {
    match handle_connection_with_error_propagating(
        peer_map,
        raw_stream,
        connected_peer_address,
        period,
    )
    .await
    {
        Ok(_) => return,
        Err(error) => {
            println!("Error during connection handling: {}", error);
            return;
        }
    };
}

async fn handle_connection_with_error_propagating(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    connected_peer_address: SocketAddr,
    period: u32,
) -> Result<()> {
    println!("Incoming TCP connection from: {}", connected_peer_address);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .context("Websocket handshake error")?;

    println!(
        "WebSocket connection established: {}",
        connected_peer_address
    );

    let (outgoing, mut incoming) = ws_stream.split();
    let (tx, rx) = unbounded();

    let connected_peer_address =
        receive_connected_peer_address(&mut incoming, connected_peer_address)
            .await
            .context("Unable to receive other peer server port")?;

    let current_peer_map = peer_map
        .lock()
        .iter()
        .map(|address| address.to_string())
        .join(" ");
    send_other_peers(tx.clone(), current_peer_map)?;
    // Insert the write part of this peer to the peer map.
    peer_map.lock().push(connected_peer_address);
    // Delete disconnected peer no matter what was happened with current connection
    let _guard = scopeguard::guard((), |_| {
        peer_map
            .lock()
            .retain(|&addr| addr != connected_peer_address);
    });

    tokio::spawn(gossiping(tx, period));

    let incoming_stream = incoming.try_for_each(|message| {
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

    // The way tokio_tungstenite offer to send messages vie websocket
    let gossiping_channel = rx.map(Ok).forward(outgoing);
    pin_mut!(incoming_stream, gossiping_channel);
    future::select(incoming_stream, gossiping_channel).await;

    println!("{} disconnected", &connected_peer_address);

    Ok(())
}

pub async fn run_server(server_address: SocketAddr, period: u32) -> Result<()> {
    let peers_to_connect = PeerMap::new(Mutex::new(Vec::new()));

    let try_socket = TcpListener::bind(&server_address).await;
    let listener = try_socket.context("Failed to bind")?;
    println!("Listening on: {}", server_address);

    // Spawn the handling of each connection in a separate task
    while let Ok((stream, connected_peer_address)) = listener.accept().await {
        tokio::spawn(handle_connection(
            peers_to_connect.clone(),
            stream,
            connected_peer_address,
            period,
        ));
    }

    Ok(())
}
