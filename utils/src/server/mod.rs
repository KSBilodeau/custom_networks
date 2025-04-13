use crate::{Connection, ConnectionType};
use anyhow::{Context, Result};
use etherparse::IpNumber;
use rustix::net::{recvfrom, sendto, AddressFamily, RecvFlags, SendFlags, SocketType};
use std::net::SocketAddrV4;
use std::os::fd::OwnedFd;

#[derive(Debug)]
pub struct CustomSocket {
    src_addr: SocketAddrV4,
    socket: OwnedFd,
}

impl CustomSocket {
    pub fn new(src_addr: SocketAddrV4) -> Result<CustomSocket> {
        let socket = rustix::net::socket(
            AddressFamily::INET,
            SocketType::RAW,
            Some(rustix::net::Protocol::from_raw(
                rustix::net::RawProtocol::new(253)
                    .with_context(|| "Failed to create custom protocol number")?,
            )),
        )
        .with_context(|| "Failed to create raw socket")?;

        Ok(CustomSocket { src_addr, socket })
    }

    pub(crate) fn send(&self, msg: &[u8], dst_addr: SocketAddrV4) -> Result<()> {
        sendto(&self.socket, msg, SendFlags::empty(), &dst_addr)?;

        Ok(())
    }

    pub(crate) fn recv(&self, ip_check: bool) -> Result<Vec<u8>> {
        let mut buf = [0u8; 65535];

        loop {
            let (bytes_read, _, _) = recvfrom(&self.socket, &mut buf, RecvFlags::empty())?;
            let remaining_packet = &buf[0..bytes_read];

            let Ok((ip_header, remaining_packet)) =
                etherparse::Ipv4Header::from_slice(remaining_packet)
            else {
                continue;
            };

            if String::from_utf8_lossy(&remaining_packet[0..self.src_addr.to_string().len()])
                == self.src_addr.to_string()
                || !ip_check
            {
                if ip_header.protocol == IpNumber::from(255) {
                    return Ok(remaining_packet.to_vec());
                }
            }
        }
    }
}

pub fn accept(socket: &CustomSocket) -> Result<Connection> {
    let broadcast_bytes = socket.recv(true)?;
    let broadcast = String::from_utf8_lossy(&*broadcast_bytes);

    let (_, client_addr) = broadcast
        .split_once("::")
        .with_context(|| "Failed to extract client IP address from broadcast")?;

    let client_socket: SocketAddrV4 = client_addr
        .parse()
        .with_context(|| "Failed to parse client IP address")?;

    socket.send(
        format!("{}::{}", client_socket, socket.src_addr).as_bytes(),
        client_socket,
    )?;

    Ok(Connection {
        conn_type: ConnectionType::Server,
        src_addr: socket.src_addr,
        dst_addr: client_socket,
        socket: &socket.socket,
    })
}
