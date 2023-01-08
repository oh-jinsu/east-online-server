use std::error::Error;

use crate::net::packet;

pub fn get_packet_buf(packet: packet::Outgoing) -> Result<Vec<u8>, Box<dyn Error>> {
    let buf = packet.serialize();

    let size: u16 = match buf.len().try_into() {
        Ok(size) => size,
        Err(_) => return Err("outgoing packet too large".into()),
    };

    Ok([&u16::to_le_bytes(size) as &[u8], &buf].concat())
}
