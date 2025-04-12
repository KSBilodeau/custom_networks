use anyhow::{bail, Context};

#[derive(Debug)]
pub(crate) enum CustomTcpFlags {
    Syn,
    Ack,
    // Fin,
}

#[derive(Debug)]
pub(crate) struct CustomTcpHeader {
    src_port: u16,
    dst_port: u16,
    seq_no: u32,
    ack_flag: bool,
    syn_flag: bool,
    fin_flag: bool,
    payload_size: u16,
}

impl CustomTcpHeader {
    fn new(src_port: u16, dst_port: u16, flags: Vec<CustomTcpFlags>) -> CustomTcpHeader {
        let mut header = CustomTcpHeader {
            src_port,
            dst_port,
            seq_no: 0,
            ack_flag: false,
            syn_flag: true,
            fin_flag: false,
            payload_size: 0,
        };

        for flag in flags {
            match flag {
                CustomTcpFlags::Syn => header.syn_flag = true,
                CustomTcpFlags::Ack => header.ack_flag = true,
                // CustomTcpFlags::Fin => header.fin_flag = true,
            }
        }

        header
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

    fn try_from(packet: &[u8]) -> anyhow::Result<Self, Self::Error> {
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

#[derive(Debug)]
pub(crate) struct CustomTcpPayload {
    header: CustomTcpHeader,
    data: [u8; CustomTcpPayload::MAX_SEGMENT_SIZE],
}

impl CustomTcpPayload {
    const MAX_SEGMENT_SIZE: usize = 1460;

    pub(crate) fn new(
        src_port: u16,
        dst_port: u16,
        flags: Vec<CustomTcpFlags>,
    ) -> CustomTcpPayload {
        CustomTcpPayload {
            header: CustomTcpHeader::new(src_port, dst_port, flags),
            data: [0u8; Self::MAX_SEGMENT_SIZE],
        }
    }

    pub(crate) fn src_port(&self) -> u16 {
        self.header.src_port
    }

    pub(crate) fn dst_port(&self) -> u16 {
        self.header.dst_port
    }

    pub(crate) fn has_syn(&self) -> bool {
        self.header.syn_flag
    }

    pub(crate) fn has_ack(&self) -> bool {
        self.header.ack_flag
    }

    // pub(crate) fn has_fin(&self) -> bool {
    //     self.header.fin_flag
    // }

    pub(crate) fn into_vec(self) -> Vec<u8> {
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

    fn try_from(packet: &[u8]) -> anyhow::Result<Self, Self::Error> {
        Ok(CustomTcpPayload {
            header: packet.try_into()?,
            data: packet[CustomTcpHeader::size()..]
                .try_into()
                .with_context(|| "Failed to convert payload bytes into slice")?,
        })
    }
}
