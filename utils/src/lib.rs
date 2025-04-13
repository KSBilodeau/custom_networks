#![deny(clippy::pedantic)]

pub mod client;
pub mod server;
mod tcp;

use crate::tcp::{CustomTcpFlags, CustomTcpPayload};
use anyhow::{bail, Context, Result};
use rustix::net::{recvfrom, sendto, RecvFlags, SendFlags};
use std::net::SocketAddrV4;
use std::os::fd::OwnedFd;

#[derive(Debug)]
enum ConnectionType {
    Server,
    Client,
}

#[derive(Debug)]
pub struct Connection<'a> {
    conn_type: ConnectionType,
    src_addr: SocketAddrV4,
    dst_addr: SocketAddrV4,
    socket: &'a OwnedFd,
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
        let payload = CustomTcpPayload::new(self.src_addr.port(), self.dst_addr.port(), msg, flags);

        sendto(
            self.socket,
            &payload.into_vec(),
            SendFlags::empty(),
            &self.dst_addr,
        )?;

        Ok(())
    }

    pub fn recv(&self) -> Result<Vec<u8>> {
        todo!()
    }

    fn recv_payload(&self) -> Result<CustomTcpPayload> {
        let mut packet_buf = [0u8; 65535];

        loop {
            let (bytes_read, _, _) = recvfrom(&self.socket, &mut packet_buf, RecvFlags::empty())
                .with_context(|| "Failed to read payload from buffer")?;
            let syn_packet = &packet_buf[0..bytes_read];

            if let Ok((ip_header, remaining_packet)) =
                etherparse::Ipv4Header::from_slice(&syn_packet)
            {
                // Check if it came from a raw socket first
                if ip_header.protocol == etherparse::IpNumber::IPV4 {
                    let payload: Result<CustomTcpPayload> = remaining_packet.try_into();

                    // Then check if it is a valid payload (could be chatter on the line or localhost
                    // loopback interference)
                    if let Ok(payload) = payload {
                        if payload.src_port() == self.dst_addr.port()
                            && payload.dst_port() == self.src_addr.port()
                        {
                            return Ok(payload);
                        }
                    }
                }
            }
        }
    }
}
