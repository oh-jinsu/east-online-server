use tokio::net::TcpStream;

use crate::net::packet;

pub enum Job {
    Accept(TcpStream),
    Drop(usize),
    Readable(usize),
    Incoming(packet::Incoming),
}
