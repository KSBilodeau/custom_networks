#![deny(clippy::pedantic)]

use anyhow::Result;
use std::net::SocketAddrV4;

fn main() -> Result<()> {
    let server_ip: SocketAddrV4 = std::env::var("SERVER_IP_ADDR")?.parse()?;

    let socket = utils::server::CustomSocket::new(server_ip)?;

    loop {
        let conn = utils::server::accept(&socket)?;

        println!("Accept completed successfully!");

        conn.handshake()?;

        println!("Handshake completed successfully!");
    }
}
