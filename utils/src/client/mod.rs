use crate::{Connection, ConnectionType};
use anyhow::{bail, Context};
use rustix::net::{recvfrom, sendto, AddressFamily, RecvFlags, SendFlags, SocketType};
use std::net::SocketAddrV4;
use std::os::fd::OwnedFd;

pub struct CustomSocket {
    src_addr: SocketAddrV4,
    dst_addr: SocketAddrV4,
    socket: OwnedFd,
}

impl CustomSocket {
    pub fn new(src_addr: SocketAddrV4, dst_addr: SocketAddrV4) -> anyhow::Result<CustomSocket> {
        let socket = rustix::net::socket(
            AddressFamily::INET,
            SocketType::RAW,
            Some(rustix::net::Protocol::from_raw(
                rustix::net::RawProtocol::new(253)
                    .with_context(|| "Failed to create custom protocol number")?,
            )),
        )
        .with_context(|| "Failed to create raw socket")?;

        Ok(CustomSocket {
            src_addr,
            dst_addr,
            socket,
        })
    }

    fn send(&self, msg: &[u8]) -> anyhow::Result<()> {
        sendto(&self.socket, msg, SendFlags::empty(), &self.dst_addr)?;

        Ok(())
    }

    fn recv(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = [0u8; 65535];

        loop {
            let (bytes_read, _, _) = recvfrom(&self.socket, &mut buf, RecvFlags::empty())?;
            let remaining_packet = &buf[0..bytes_read];

            let Ok((_, remaining_packet)) = etherparse::Ipv4Header::from_slice(remaining_packet)
            else {
                continue;
            };

            if String::from_utf8_lossy(&remaining_packet[0..self.src_addr.to_string().len()])
                == self.src_addr.to_string()
            {
                return Ok(remaining_packet.to_vec());
            }
        }
    }
}

pub fn connect(socket: &CustomSocket) -> anyhow::Result<Connection> {
    let broadcast_string = format!("{}::{}", socket.dst_addr, socket.src_addr);

    socket.send(&broadcast_string.as_bytes())?;

    let broadcast_echo = socket.recv()?;

    if String::from_utf8_lossy(&broadcast_echo)
        != format!("{}::{}", socket.src_addr, socket.dst_addr)
    {
        bail!("Incorrect broadcast echo message")
    }

    Ok(Connection {
        conn_type: ConnectionType::Client,
        src_addr: socket.src_addr,
        dst_addr: socket.dst_addr,
        socket: &socket.socket,
    })
}
