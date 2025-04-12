#![deny(clippy::pedantic)]

mod client;
mod server;
mod tcp;

use crate::tcp::CustomTcpPayload;
use anyhow::{Context, Result};
use etherparse::IpNumber;
use rustix::fd::OwnedFd;
use rustix::io::read;
use rustix::net::{bind, sendto, socket, AddressFamily, SendFlags, SocketType};
use std::net::SocketAddr;

#[derive(Debug)]
pub enum ConnectionType {
    Server,
    Client,
}

pub fn bind_raw(ip_addr: &str) -> Result<OwnedFd> {
    let socket_file_desc = create_socket()?;
    let sock_addr: SocketAddr = format!("{}:0000", ip_addr)
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

pub fn handshake(
    fd: &OwnedFd,
    ip_addr: &str,
    src_port: &str,
    dst_port: Option<&str>,
    conn_type: ConnectionType,
) -> Result<()> {
    match conn_type {
        ConnectionType::Server => server::server_handshake(fd, ip_addr, src_port)?,
        ConnectionType::Client => client::client_handshake(fd, ip_addr, src_port, dst_port)?,
    };

    Ok(())
}

fn send(fd: &OwnedFd, payload: CustomTcpPayload, sock_addr: &SocketAddr) -> Result<()> {
    sendto(&fd, &payload.into_vec(), SendFlags::empty(), sock_addr)
        .with_context(|| "Failed to write to socket")?;

    Ok(())
}

fn recv(fd: &OwnedFd, dst_port: Option<u16>) -> Result<CustomTcpPayload> {
    let payload: CustomTcpPayload;

    loop {
        let mut packet_buf = [0u8; 65535];

        let bytes_read = read(&fd, &mut packet_buf)?;
        let syn_packet = &packet_buf[0..bytes_read];

        let (ip_header, remaining_packet) = etherparse::Ipv4Header::from_slice(&syn_packet)?;

        if ip_header.protocol == IpNumber::from(255) {
            let temp: CustomTcpPayload = remaining_packet.try_into()?;

            if let Some(port) = dst_port {
                if temp.dst_port() == port {
                    payload = temp;
                    break;
                }
            } else {
                payload = temp;
                break;
            }
        }
    }

    Ok(payload)
}
