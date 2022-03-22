use std::{collections::HashMap, net::SocketAddr, sync::Mutex};

use crate::{gossiping::gossiping, PeerMap};
use futures::{future, pin_mut, StreamExt, TryStreamExt};
use futures_channel::mpsc::unbounded;
use tokio::net::{TcpListener, TcpStream};

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

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    dbg!(&addr.ip().to_string());
    peer_map.lock().unwrap().insert(addr, tx.clone());

    let (outgoing, mut incoming) = ws_stream.split();

    tokio::spawn(gossiping(tx, period));

    let connected_peer_listenting_port = incoming.next().await.unwrap().unwrap().to_string();
    let connected_peer_address =
        SocketAddr::new(addr.ip(), connected_peer_listenting_port.parse().unwrap());
    dbg!(&connected_peer_address);

    let broadcast_incoming = incoming.try_for_each(|msg| {
        println!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);
    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

pub async fn run_server(port: u16, period: u32) {
    let addr = format!("127.0.0.1:{}", port);

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr, period));
    }
}
