use east_online_core::model::Vector3;
use tokio::{net::TcpStream, time};

use crate::net::packet;

pub enum Job {
    Accept(TcpStream, String, Vector3),
    Drop(String, String),
    Readable(String),
    Incoming(String, packet::Incoming),
    Write(String, packet::Outgoing),
    Broadcast(packet::Outgoing),
    Move(String, time::Duration),
}
