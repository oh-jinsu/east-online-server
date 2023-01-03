use std::error::Error;
use tokio::net::TcpListener;

pub struct Gatekeeper {
    listener: TcpListener,
}

impl Gatekeeper {
    pub fn new(listener: TcpListener) -> Self {
        Gatekeeper { listener }
    }

    pub async fn keep(&self) -> Result<(), Box<dyn Error>> {
        loop {}
    }
}
