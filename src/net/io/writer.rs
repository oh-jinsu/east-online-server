use std::io;

use tokio::net::TcpStream;

pub trait Writer {
    fn try_write_one(&self, buf: &mut Vec<u8>) -> io::Result<()>;

    fn try_write_to_end(&self, buf: &mut [u8]) -> io::Result<()>;
}

impl Writer for TcpStream {
    fn try_write_one(&self, buf: &mut Vec<u8>) -> io::Result<()> {
        let size: u16 = match buf.len().try_into() {
            Ok(size) => size,
            Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "buffer too large")),
        };

        let mut buf = [&u16::to_le_bytes(size) as &[u8], buf].concat();

        self.try_write_to_end(&mut buf)
    }

    fn try_write_to_end(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut pos = 0;

        while pos < buf.len() {
            match self.try_write(&mut buf[pos..]) {
                Ok(0) => return Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
                Ok(n) => {
                    pos += n;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
