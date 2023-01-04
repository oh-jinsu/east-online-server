use tokio::net::TcpStream;

pub enum Job {
    Accept(TcpStream),
}
