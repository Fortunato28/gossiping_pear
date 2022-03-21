use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::{stream::SplitSink, SinkExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{http::uri::Port, protocol::Message};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
fn nothing_to_do() {}

async fn gossiping(tx: UnboundedSender<Message>, addr: SocketAddr) {
    loop {
        // TODO make random message
        match tx.unbounded_send(Message::Text("test".to_string())) {
            Ok(_) => nothing_to_do(),
            Err(_) => return,
        }

        dbg!(&"Still working");

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx.clone());

    let (outgoing, incoming) = ws_stream.split();

    tokio::spawn(gossiping(tx, addr.clone()));

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

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    // TODO Figure out what means arguments below
    #[clap(short, long)]
    period: u32,
    #[clap(long)]
    port: u16,
    // TODO make a Vec<Uri>
    #[clap(long)]
    connection: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let arguments = Arguments::parse();
    dbg!(&arguments);

    let addr = "127.0.0.1:8080".to_string();

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
