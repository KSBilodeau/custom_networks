#![deny(clippy::pedantic)]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let ip_addr =
        std::env::var("IP_ADDR").with_context(|| "Missing IP addr environment variable")?;
    let src_port =
        std::env::var("SRC_PORT").with_context(|| "Missing port environment variable")?;

    let socket_file_desc = utils::bind_raw(&ip_addr)?;

    loop {
        utils::handshake(
            &socket_file_desc,
            &ip_addr,
            &src_port,
            None,
            utils::ConnectionType::Server,
        )?;

        println!("Handshake completed successfully!");
    }
}
