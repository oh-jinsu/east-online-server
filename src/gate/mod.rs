use std::error::Error;
use tokio::net::TcpListener;

pub struct Keeper {
    listener: TcpListener,
}

impl Keeper {
    pub fn new(listener: TcpListener) -> Self {
        Keeper { listener }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        loop {}
    }
}
