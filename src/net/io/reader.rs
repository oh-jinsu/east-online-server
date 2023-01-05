use std::io;

use tokio::net::TcpStream;

use crate::net::packet;

pub trait Reader {
    fn try_read_packet(&self) -> io::Result<packet::Incoming>;
}

impl Reader for TcpStream {
    fn try_read_packet(&self) -> io::Result<packet::Incoming> {
        let mut buf = Vec::with_capacity(2);

        self.try_read_buf(&mut buf)?;

        if buf.len() < 2 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        let size = usize::from(u16::from_le_bytes([buf[0], buf[1]]));

        if size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("zero size packet, {}", size),
            ));
        }

        if size > 8096 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("too large packet, {}", size),
            ));
        }

        let mut buf = Vec::with_capacity(size);

        self.try_read_buf(&mut buf)?;

        packet::Incoming::deserialize(&mut buf)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
    }
}
