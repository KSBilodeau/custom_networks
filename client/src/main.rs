#![deny(clippy::pedantic)]

use anyhow::Result;
use std::net::SocketAddrV4;

fn main() -> Result<()> {
    let server_ip: SocketAddrV4 = std::env::var("SERVER_IP_ADDR")?.parse()?;
    let client_ip: SocketAddrV4 = std::env::var("CLIENT_IP_ADDR")?.parse()?;

    let socket = utils::client::CustomSocket::new(client_ip, server_ip)?;

    let conn = utils::client::connect(&socket)?;

    println!("Connect completed successfully!");

    conn.handshake()?;

    println!("Handshake completed successfully!");

    Ok(())
}
