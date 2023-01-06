use tokio::net::TcpStream;

use crate::net::packet;

pub enum Job {
    Accept(TcpStream),
    Drop(usize, String),
    Readable(usize),
    Incoming(usize, packet::Incoming),
    Send(usize, String, String),
}
