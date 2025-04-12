use crate::{Connection, ConnectionType};
use anyhow::Context;
use rustix::io::read;
use rustix::net::sendto;
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, OwnedFd};

pub fn connect<'a>(
    socket: &'a OwnedFd,
    src_port: &str,
    dst_ip_addr: &str,
    dst_port: &str,
) -> anyhow::Result<Connection<'a>> {
    unsafe {
        libc::setsockopt(
            socket.as_raw_fd(),
            libc::IPPROTO_RAW,
            libc::IP_HDRINCL,
            &0 as *const _ as *const _,
            4,
        );
    }

    // Create server socket address
    let server_addr: SocketAddr = format!("{}:{}", dst_ip_addr, dst_port)
        .parse()
        .with_context(|| "Failed to convert ip address from string")?;

    // Initiate connection with the server by broadcasting address to it
    sendto(
        &socket,
        &format!("{}::{}", dst_port, src_port).into_bytes(),
        rustix::net::SendFlags::empty(),
        &server_addr,
    )?;

    // Wait for server accept message
    let mut buf = [0u8; 65535];
    loop {
        let bytes_read =
            read(socket, &mut buf).with_context(|| "Failed to read bytes from socket")?;
        
        if let Ok((ip_header, buf)) = etherparse::Ipv4Header::from_slice(&buf[0..bytes_read]) {
            if ip_header.protocol == etherparse::IpNumber::IPV4
                && buf == format!("{}::{}", &src_port, &dst_port).as_bytes()
            {
                break;
            }
        }
    }

    Ok(Connection {
        conn_type: ConnectionType::Client,
        src_socket: &socket,
        src_port: src_port
            .parse()
            .with_context(|| "Failed to convert port to u16")?,
        dst_socket: server_addr,
        dst_port: dst_port
            .parse()
            .with_context(|| "Failed to convert port to u16")?,
    })
}
