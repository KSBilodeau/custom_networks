#![deny(clippy::pedantic)]

use std::net::SocketAddr;
use anyhow::{Context, Result};
use rustix::fd::OwnedFd;
use rustix::net::{bind, socket, AddressFamily, SocketType};

#[allow(dead_code)]
pub struct CustomTcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_no: u32,
    ack_flag: bool,
    syn_flag: bool,
    fin_flag: bool,
    payload_size: u16,
}

impl CustomTcpHeader {
    const fn size() -> usize {
        3 * size_of::<u16>() + size_of::<u32>() + 3 * size_of::<bool>()
    }
}

impl From<&CustomTcpHeader> for Vec<u8> {
    fn from(header: &CustomTcpHeader) -> Self {
        let mut result = Vec::with_capacity(CustomTcpPayload::size());

        result.extend_from_slice(&header.src_port.to_be_bytes());
        result.extend_from_slice(&header.dst_port.to_be_bytes());
        result.extend_from_slice(&header.seq_no.to_be_bytes());
        result.push(header.ack_flag as u8);
        result.push(header.syn_flag as u8);
        result.push(header.fin_flag as u8);
        result.extend_from_slice(&header.payload_size.to_be_bytes());

        result
    }
}

#[allow(dead_code)]
pub struct CustomTcpPayload {
    header: CustomTcpHeader,
    data: [u8; u16::MAX as usize],
}

impl CustomTcpPayload {
    const fn size() -> usize {
        CustomTcpHeader::size() + size_of::<u8>() * u16::MAX as usize
    }
}

impl From<&CustomTcpPayload> for Vec<u8> {
    fn from(payload: &CustomTcpPayload) -> Self {
        let mut result = Vec::with_capacity(CustomTcpPayload::size());

        result.extend_from_slice(&Vec::<u8>::from(&payload.header));
        result.extend_from_slice(&payload.data);

        result
    }
}

pub enum ConnectionType {
    Server,
    Client,
}

pub fn bind_raw(ip_addr: &str) -> Result<OwnedFd> {
    let socket_file_desc = create_socket()?;
    let sock_addr: SocketAddr = format!("{}:0000", ip_addr).parse()?;
    
    bind(&socket_file_desc, &sock_addr)?;

    Ok(socket_file_desc)
}

pub fn handshake(fd: &OwnedFd, conn_type: ConnectionType) -> Result<()> {
    match conn_type {
        ConnectionType::Server => server_handshake(fd)?,
        ConnectionType::Client => client_handshake(fd)?,
    };

    Ok(())
}

fn create_socket() -> Result<OwnedFd> {
    socket(
        AddressFamily::INET,
        SocketType::RAW,
        Some(rustix::net::ipproto::RAW),
    )
    .with_context(|| "Failed to create socket")
}

fn server_handshake(_fd: &OwnedFd) -> Result<()> {
    Ok(())
}

fn client_handshake(_fd: &OwnedFd) -> Result<()> {
    Ok(())
}
