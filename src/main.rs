pub mod client;
pub mod gossiping;
pub mod server;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use clap::Parser;
use parking_lot::Mutex;

// Using parking_lot::Mutex to avoid poisoning which is not matter in this implementation
type PeerMap = Arc<Mutex<Vec<SocketAddr>>>;

fn nothing_to_do() {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Arguments {
    #[clap(long)]
    period: u32,
    #[clap(long)]
    port: u16,
    #[clap(long)]
    connection: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Arguments {
        period,
        port,
        connection,
    } = Arguments::parse();

    let server_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    client::run(period, &connection, server_address);
    server::run(server_address, period).await?;

    Ok(())
}
