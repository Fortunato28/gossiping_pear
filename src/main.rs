pub mod client;
pub mod gossiping;
pub mod server;

use std::{
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use clap::Parser;

use client::run_client;
use server::run_server;

type PeerMap = Arc<Mutex<Vec<SocketAddr>>>;

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

    run_client(period, connection, port);
    run_server(port, period).await;

    Ok(())
}
