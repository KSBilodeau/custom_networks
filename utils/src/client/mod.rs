use std::net::SocketAddr;
use std::os::fd::OwnedFd;
use anyhow::{bail, Context};
use crate::{recv, send};
use crate::tcp::{CustomTcpFlags, CustomTcpPayload};

pub(crate) fn client_handshake(
    fd: &OwnedFd,
    ip_addr: &str,
    src_port: &str,
    dst_port: Option<&str>,
) -> anyhow::Result<()> {
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

    send(
        fd,
        CustomTcpPayload::new(src_port, dst_port, vec![CustomTcpFlags::Syn]),
        &sock_addr,
    )?;

    let syn_ack_payload = recv(fd, Some(src_port))?;

    if syn_ack_payload.has_syn() && syn_ack_payload.has_ack() {
        send(
            fd,
            CustomTcpPayload::new(src_port, dst_port, vec![CustomTcpFlags::Ack]),
            &sock_addr,
        )?;
    } else {
        bail!("Handshake missing syn-ack flags");
    }

    Ok(())
}