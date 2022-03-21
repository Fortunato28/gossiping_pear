pub mod client;
pub mod server;

use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use clap::Parser;
use futures_channel::mpsc::UnboundedSender;
use tungstenite::protocol::Message;

use client::run_client;
use server::run_server;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

fn nothing_to_do() {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    // TODO Figure out what means arguments below
    #[clap(short, long)]
    period: u32,
    #[clap(long)]
    port: u16,
    // TODO make a Vec<Uri>
    // TODO change name of field
    #[clap(long)]
    connection: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let Arguments {
        period,
        port,
        connection,
    } = Arguments::parse();

    run_client(period, connection);
    run_server(port).await;

    Ok(())
}
