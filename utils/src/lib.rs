#![deny(clippy::pedantic)]

pub mod client;
pub mod server;
mod tcp;

use crate::tcp::CustomTcpPayload;
use anyhow::{Context, Result};
use rustix::fd::OwnedFd;
use rustix::net::{bind, socket, AddressFamily, SocketType};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct Connection<'a> {
    src_socket: &'a OwnedFd,
    src_port: u16,
    dst_socket: SocketAddr,
    dst_port: u16,
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

fn send(fd: &OwnedFd, payload: CustomTcpPayload, dst_addr: &SocketAddr) -> Result<()> {
    todo!()
}

fn recv(fd: &OwnedFd, dst_port: u16) -> Result<CustomTcpPayload> {
    todo!()
}
