#![deny(clippy::pedantic)]

pub mod client;
pub mod server;
mod tcp;

use crate::tcp::{CustomTcpFlags, CustomTcpPayload};
use anyhow::{bail, Context, Result};
use rustix::fd::OwnedFd;
use rustix::io::read;
use rustix::net::{bind, sendto, socket, AddressFamily, SocketType};
use std::net::SocketAddr;

#[derive(Debug)]
enum ConnectionType {
    Server,
    Client,
}

#[derive(Debug)]
pub struct Connection<'a> {
    conn_type: ConnectionType,
    src_socket: &'a OwnedFd,
    src_port: u16,
    dst_socket: SocketAddr,
    dst_port: u16,
}

impl Connection<'_> {
    pub fn handshake(&self) -> Result<()> {
        match self.conn_type {
            ConnectionType::Server => self.handshake_server(),
            ConnectionType::Client => self.handshake_client(),
        }
    }

    fn handshake_server(&self) -> Result<()> {
        let payload = self.recv_payload()?;

        if payload.has_syn() {
            self.send_with_tcp_flags(b"", vec![CustomTcpFlags::Syn, CustomTcpFlags::Ack])?;

            let payload = self.recv_payload()?;

            if !payload.has_ack() {
                bail!("Missing final ack payload!");
            }
        } else {
            bail!("Missing initial syn payload!");
        }

        Ok(())
    }

    fn handshake_client(&self) -> Result<()> {
        self.send_with_tcp_flags(b"", vec![CustomTcpFlags::Syn])?;

        let payload = self.recv_payload()?;

        if payload.has_syn() && payload.has_ack() {
            self.send_with_tcp_flags(b"", vec![CustomTcpFlags::Ack])?;
        } else {
            bail!("Missing syn-ack response from server!");
        }

        Ok(())
    }

    pub fn send(&self, msg: &[u8]) -> Result<()> {
        self.send_with_tcp_flags(msg, vec![])
    }

    fn send_with_tcp_flags(&self, msg: &[u8], flags: Vec<CustomTcpFlags>) -> Result<()> {
        let payload = CustomTcpPayload::new(self.src_port, self.dst_port, msg, flags);

        sendto(
            &self.src_socket,
            &payload.into_vec(),
            rustix::net::SendFlags::empty(),
            &self.dst_socket,
        )
        .with_context(|| "Failed to write to socket")?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Vec<u8>> {
        todo!()
    }

    fn recv_payload(&self) -> Result<CustomTcpPayload> {
        let mut packet_buf = [0u8; 65535];

        loop {
            let bytes_read = read(&self.src_socket, &mut packet_buf)
                .with_context(|| "Failed to read payload from buffer")?;
            let syn_packet = &packet_buf[0..bytes_read];

            let (ip_header, remaining_packet) = etherparse::Ipv4Header::from_slice(&syn_packet)
                .with_context(|| "Failed to extract IpV4 header")?;

            // Check if it came from a raw socket first
            if ip_header.protocol == etherparse::IpNumber::from(255) {
                let payload: Result<CustomTcpPayload> = remaining_packet.try_into();
                
                // Then check if it is a valid payload (could be chatter on the line or localhost
                // loopback interference)
                if let Ok(payload) = payload {
                    if payload.src_port() == self.dst_port && payload.dst_port() == self.src_port {
                        return Ok(payload);
                    }
                }
            }
        }
    }
}

pub fn bind_raw(ip_addr: &str, port: &str) -> Result<OwnedFd> {
    let socket_file_desc = create_socket()?;
    let sock_addr: SocketAddr = format!("{}:{}", ip_addr, port)
        .parse()
        .with_context(|| "Failed to convert ip address from string")?;

    bind(&socket_file_desc, &sock_addr)?;

    Ok(socket_file_desc)
}

fn create_socket() -> Result<OwnedFd> {
    socket(
        AddressFamily::INET,
        SocketType::RAW,
        Some(rustix::net::ipproto::RAW),
    )
    .with_context(|| "Failed to create socket")
}
