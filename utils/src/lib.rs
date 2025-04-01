#![deny(clippy::pedantic)]

use anyhow::{bail, Context, Result};
use etherparse::IpNumber;
use rustix::fd::OwnedFd;
use rustix::io::read;
use rustix::net::{bind, sendto, socket, AddressFamily, SendFlags, SocketType};
use std::net::SocketAddr;

#[allow(dead_code)]
#[derive(Debug)]
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
    fn syn(src_port: u16, dst_port: u16) -> CustomTcpHeader {
        CustomTcpHeader {
            src_port,
            dst_port,
            seq_no: 0,
            ack_flag: false,
            syn_flag: true,
            fin_flag: false,
            payload_size: 0,
        }
    }

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

impl TryFrom<&[u8]> for CustomTcpHeader {
    type Error = anyhow::Error;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        Ok(CustomTcpHeader {
            src_port: u16::from_be_bytes(
                packet[0..2]
                    .try_into()
                    .with_context(|| "Failed to convert bytes into slice")?,
            ),
            dst_port: u16::from_be_bytes(
                packet[2..4]
                    .try_into()
                    .with_context(|| "Failed to convert bytes into slice")?,
            ),
            seq_no: u32::from_be_bytes(
                packet[4..8]
                    .try_into()
                    .with_context(|| "Failed to convert bytes into slice")?,
            ),
            ack_flag: match packet[8] {
                0 => false,
                1 => true,
                _ => bail!("Failed to convert byte into bool"),
            },
            syn_flag: match packet[9] {
                0 => false,
                1 => true,
                _ => bail!("Failed to convert byte into bool"),
            },
            fin_flag: match packet[10] {
                0 => false,
                1 => true,
                _ => bail!("Failed to convert byte into bool"),
            },
            payload_size: u16::from_be_bytes(
                packet[11..13]
                    .try_into()
                    .with_context(|| "Failed to convert bytes into slice")?,
            ),
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CustomTcpPayload {
    header: CustomTcpHeader,
    data: [u8; CustomTcpPayload::MAX_SEGMENT_SIZE],
}

impl CustomTcpPayload {
    const MAX_SEGMENT_SIZE: usize = 1460;

    fn syn(src_port: u16, dst_port: u16) -> CustomTcpPayload {
        CustomTcpPayload {
            header: CustomTcpHeader::syn(src_port, dst_port),
            data: [0u8; Self::MAX_SEGMENT_SIZE],
        }
    }

    fn ack(src_port: u16, dst_port: u16) -> CustomTcpPayload {
        let mut payload = Self::syn(src_port, dst_port);
        payload.header.syn_flag = false;
        payload.header.ack_flag = true;

        payload
    }

    fn syn_ack(src_port: u16, dst_port: u16) -> CustomTcpPayload {
        let mut payload = Self::syn(src_port, dst_port);
        payload.header.ack_flag = true;

        payload
    }

    fn into_vec(self) -> Vec<u8> {
        Vec::<u8>::from(self)
    }

    const fn size() -> usize {
        CustomTcpHeader::size() + size_of::<u8>() * Self::MAX_SEGMENT_SIZE
    }
}

impl From<CustomTcpPayload> for Vec<u8> {
    fn from(payload: CustomTcpPayload) -> Self {
        let mut result = Vec::with_capacity(CustomTcpPayload::size());

        result.extend_from_slice(&Vec::<u8>::from(&payload.header));
        result.extend_from_slice(&payload.data);

        result
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

impl TryFrom<&[u8]> for CustomTcpPayload {
    type Error = anyhow::Error;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        Ok(CustomTcpPayload {
            header: packet.try_into()?,
            data: packet[CustomTcpHeader::size()..]
                .try_into()
                .with_context(|| "Failed to convert payload bytes into slice")?,
        })
    }
}

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

pub fn handshake(fd: &OwnedFd, ip_addr: &str, src_port: &str, dst_port: Option<&str>, conn_type: ConnectionType) -> Result<()> {
    match conn_type {
        ConnectionType::Server => server_handshake(fd, ip_addr, src_port)?,
        ConnectionType::Client => client_handshake(fd, ip_addr, src_port, dst_port)?,
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

fn server_handshake(fd: &OwnedFd, ip_addr: &str, src_port: &str) -> Result<()> {
    let sock_addr: SocketAddr = format!("{}:0000", ip_addr)
        .parse()
        .with_context(|| "Failed to convert ip address from string")?;

    let src_port = src_port
        .parse()
        .with_context(|| "Failed to convert port to u16")?;

    let syn_payload = recv(fd, None)?;

    let dst_port = syn_payload.header.src_port;

    if syn_payload.header.syn_flag {
        send(fd, CustomTcpPayload::syn_ack(src_port, dst_port), &sock_addr)?;

        let ack_payload = recv(fd, Some(src_port))?;

        if !ack_payload.header.ack_flag {
            bail!("Handshake missing final ack flag");
        }
    } else {
        bail!("Handshake missing initial syn flag");
    }

    Ok(())
}

fn client_handshake(fd: &OwnedFd, ip_addr: &str, src_port: &str, dst_port: Option<&str>) -> Result<()> {
    let sock_addr: SocketAddr = format!("{}:0000", ip_addr)
        .parse()
        .with_context(|| "Failed to convert ip address from string")?;

    let src_port = src_port
        .parse()
        .with_context(|| "Failed to convert port to u16")?;
    let dst_port = dst_port
        .with_context(|| "Missing server port")?
        .parse()
        .with_context(|| "Failed to convert port to u16")?;

    send(fd, CustomTcpPayload::syn(src_port, dst_port), &sock_addr)?;

    let syn_ack_payload = recv(fd, Some(src_port))?;

    if syn_ack_payload.header.syn_flag && syn_ack_payload.header.ack_flag {
        send(fd, CustomTcpPayload::ack(src_port, dst_port), &sock_addr)?;
    } else {
        bail!("Handshake missing syn-ack flags");
    }

    Ok(())
}

fn send(fd: &OwnedFd, payload: CustomTcpPayload, sock_addr: &SocketAddr) -> Result<()> {
    println!("send: {:?}", payload.header);

    sendto(
        &fd,
        &payload.into_vec(),
        SendFlags::empty(),
        sock_addr,
    )
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
                if temp.header.dst_port == port {
                    payload = temp;
                    println!("recv: {:?}", payload.header);
                    break;
                }
            } else {
                payload = temp;
                println!("recv: {:?}", payload.header);
                break;
            }
        }
    }

    Ok(payload)
}
