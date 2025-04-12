use crate::{Connection, ConnectionType};
use anyhow::Context;
use rustix::io::read;
use rustix::net::sendto;
use std::net::SocketAddr;
use std::os::fd::OwnedFd;

pub fn accept<'a>(
    socket: &'a OwnedFd,
    src_ip_addr: &str,
    src_port: &str,
) -> anyhow::Result<Connection<'a>> {
    // Look for incoming IP address broadcasts from potential clients
    let mut broadcast_buf = [0u8; 65535];
    loop {
        let bytes_read = read(socket, &mut broadcast_buf)
            .with_context(|| "Failed to read broadcast bytes to buffer")?;
        let broadcast = &broadcast_buf[0..bytes_read];

        let (_, broadcast) = etherparse::Ipv4Header::from_slice(broadcast)
            .with_context(|| "Failed to construct header from buffer")?;

        let broadcast_str = String::from_utf8_lossy(&broadcast);

        let (port, client_addr) = broadcast_str
            .split_once("::")
            .with_context(|| "Failed to extract ip addr from port")?;

        let (client_addr, client_port) = client_addr
            .split_once(":")
            .with_context(|| "Failed to extract client addr from port")?;

        if src_port == port {
            let client_addr: SocketAddr = format!("{}:{}", client_addr, client_port)
                .parse()
                .with_context(|| "Failed to convert ip address from string")?;

            sendto(
                &socket,
                format!("{}::{}:{}", &client_port, &src_ip_addr, &src_port).as_bytes(),
                rustix::net::SendFlags::empty(),
                &client_addr,
            )?;

            return Ok(Connection {
                conn_type: ConnectionType::Server,
                src_socket: socket,
                src_port: src_port
                    .parse()
                    .with_context(|| "Failed to convert port to u16")?,
                dst_socket: client_addr,
                dst_port: client_port
                    .parse()
                    .with_context(|| "Failed to convert port to u16")?,
            });
        }
    }
}
