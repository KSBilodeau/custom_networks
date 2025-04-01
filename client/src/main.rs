#![deny(clippy::pedantic)]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let socket_file_desc = utils::bind_raw(
        &std::env::var("IP_ADDR").with_context(|| "Missing IP address environment variable")?,
    )?;

    utils::handshake(&socket_file_desc, utils::ConnectionType::Client)?;

    Ok(())
}
