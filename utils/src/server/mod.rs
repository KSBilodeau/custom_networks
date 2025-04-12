use crate::{Connection, ConnectionType};
use anyhow::Context;
use rustix::net::{recvfrom, sendto, RecvFlags};
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, OwnedFd};

pub fn accept<'a>(socket: &'a OwnedFd, src_port: &str) -> anyhow::Result<Connection<'a>> {
    unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::IPPROTO_RAW,
            libc::IP_HDRINCL,
            &0 as *const _ as *const _,
            4,
        );
    }
    // Look for incoming IP address broadcasts from potential clients
    let mut broadcast_buf = [0u8; 65535];
    loop {
        let (bytes_read, _, sock_addr) =
            recvfrom(socket, &mut broadcast_buf, RecvFlags::empty())
                .with_context(|| "Failed to read broadcast bytes to buffer")?;
        let broadcast = &broadcast_buf[0..bytes_read];

        let (_, broadcast) = etherparse::Ipv4Header::from_slice(broadcast)
            .with_context(|| "Failed to construct header from buffer")?;

        let broadcast_str = String::from_utf8_lossy(&broadcast);

        let (port, client_port) = broadcast_str
            .split_once("::")
            .with_context(|| "Failed to extract ip addr from port")?;

        if src_port == port {
            if let Some(client_addr) = sock_addr {
                sendto(
                    &socket,
                    format!("{}::{}", &client_port, &src_port).as_bytes(),
                    rustix::net::SendFlags::empty(),
                    &client_addr,
                )?;

                return Ok(Connection {
                    conn_type: ConnectionType::Server,
                    src_socket: socket,
                    src_port: src_port
                        .parse()
                        .with_context(|| "Failed to convert port to u16")?,
                    dst_socket: SocketAddr::try_from(client_addr)?,
                    dst_port: client_port
                        .parse()
                        .with_context(|| "Failed to convert port to u16")?,
                });
            }
        }
    }
}
