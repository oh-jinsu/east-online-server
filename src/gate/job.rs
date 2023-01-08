use tokio::net::TcpStream;

use crate::net::packet;

pub enum Job {
    Accept(TcpStream),
    Drop(usize, String),
    Readable(usize),
    Incoming(usize, packet::Incoming),
    Send {
        index: usize,
        user_id: String,
        map_id: String,
    },
}
