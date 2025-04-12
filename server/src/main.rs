#![deny(clippy::pedantic)]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let src_ip_addr =
        std::env::var("SRC_IP_ADDR").with_context(|| "Missing IP addr environment variable")?;
    let dst_ip_addr =
        std::env::var("DST_IP_ADDR").with_context(|| "Missing IP addr environment variable")?;
    let src_port =
        std::env::var("SRC_PORT").with_context(|| "Missing port environment variable")?;

    let socket_file_desc = utils::bind_raw(&src_ip_addr)?;

    loop {
        utils::handshake(
            &socket_file_desc,
            &dst_ip_addr,
            &src_port,
            None,
            utils::ConnectionType::Server,
        )?;

        println!("Handshake completed successfully!");
    }
}
