use tokio::time;

use super::Map;
use std::error::Error;

pub struct Worker {
    map: Map,
}

impl Worker {
    pub fn new(map: Map) -> Self {
        Worker { map }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        loop {}
    }
}
