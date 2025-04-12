#![deny(clippy::pedantic)]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let _src_ip_addr =
        std::env::var("SRC_IP_ADDR").with_context(|| "Missing SRC IP addr environment variable")?;
    let dst_ip_addr = std::env::var("DST_IP_ADDR")
        .with_context(|| "Missing DEST IP addr environment variable")?;
    let src_port =
        std::env::var("SRC_PORT").with_context(|| "Missing port environment variable")?;
    let dst_port =
        std::env::var("DST_PORT").with_context(|| "Missing port environment variable")?;

    let socket_file_desc = utils::create_socket()?;

    let conn = utils::client::connect(&socket_file_desc, &src_port, &dst_ip_addr, &dst_port)?;

    println!("Connect completed successfully!");

    conn.handshake()?;

    println!("Handshake completed successfully!");

    Ok(())
}
