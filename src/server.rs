use std::{net::SocketAddr, sync::Mutex};

use crate::{gossiping::gossiping, PeerMap};
use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use itertools::Itertools;
use tokio::net::{TcpListener, TcpStream};
use tungstenite::Message;

fn send_other_peers(tx: UnboundedSender<Message>, current_peer_map: String) {
    match tx.unbounded_send(Message::Text(current_peer_map)) {
        Ok(_) => dbg!(&"Peer map successfully shared"),
        Err(_) => dbg!(&"Error while trying to share peer map"),
    };
}

// TODO share contacts with new peer
async fn handle_connection(
    peer_map: PeerMap,
    raw_stream: TcpStream,
    addr: SocketAddr,
    period: u32,
) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (outgoing, mut incoming) = ws_stream.split();
    let (tx, rx) = unbounded();

    let connected_peer_listenting_port = incoming.next().await.unwrap().unwrap().to_string();
    let connected_peer_address =
        SocketAddr::new(addr.ip(), connected_peer_listenting_port.parse().unwrap());
    dbg!(&connected_peer_address);

    let current_peer_map = peer_map
        .lock()
        .unwrap()
        .iter()
        .map(|address| address.to_string())
        .join(" ");
    send_other_peers(tx.clone(), current_peer_map);
    // Insert the write part of this peer to the peer map.
    peer_map.lock().unwrap().push(connected_peer_address);

    tokio::spawn(gossiping(tx, period));

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );

        future::ok(())
    });

    let gossiping_channel = rx.map(Ok).forward(outgoing);
    pin_mut!(broadcast_incoming, gossiping_channel);
    future::select(broadcast_incoming, gossiping_channel).await;

    println!("{} disconnected", &addr);
    // Delete disconnected peer
    peer_map
        .lock()
        .unwrap()
        .retain(|&addr| addr != connected_peer_address);
}

pub async fn run_server(port: u16, period: u32) {
    let addr = format!("127.0.0.1:{}", port);

    let state = PeerMap::new(Mutex::new(Vec::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr, period));
    }
}
