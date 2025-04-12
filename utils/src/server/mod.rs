use std::net::SocketAddr;
use std::os::fd::OwnedFd;
use anyhow::{bail, Context};
use crate::{recv, send};
use crate::tcp::{CustomTcpFlags, CustomTcpPayload};

pub(crate) fn server_handshake(fd: &OwnedFd, ip_addr: &str, src_port: &str) -> anyhow::Result<()> {
    let sock_addr: SocketAddr = format!("{}:0000", ip_addr)
        .parse()
        .with_context(|| "Failed to convert ip address from string")?;

    let src_port = src_port
        .parse()
        .with_context(|| "Failed to convert port to u16")?;

    let syn_payload = recv(fd, None)?;

    let dst_port = syn_payload.src_port();

    if syn_payload.has_syn() {
        send(
            fd,
            CustomTcpPayload::new(
                src_port,
                dst_port,
                vec![CustomTcpFlags::Syn, CustomTcpFlags::Ack],
            ),
            &sock_addr,
        )?;

        let ack_payload = recv(fd, Some(src_port))?;

        if !ack_payload.has_ack() {
            bail!("Handshake missing final ack flag");
        }
    } else {
        bail!("Handshake missing initial syn flag");
    }

    Ok(())
}
