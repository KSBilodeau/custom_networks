#![deny(clippy::pedantic)]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let _src_ip_addr =
        std::env::var("SRC_IP_ADDR").with_context(|| "Missing IP addr environment variable")?;
    let src_port =
        std::env::var("SRC_PORT").with_context(|| "Missing port environment variable")?;

    let socket = utils::create_socket()?;

    loop {
        let conn = utils::server::accept(&socket, &src_port)?;

        println!("Accept completed successfully!");

        conn.handshake()?;

        println!("Handshake completed successfully!");
    }
}
