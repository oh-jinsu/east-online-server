use east_online_core::model::{self, Vector3};
use tokio::{net::TcpStream, sync::mpsc};

use super::Tile;
use std::{collections::HashMap, error::Error, sync::Arc};

type Receiver = mpsc::Receiver<(TcpStream, String)>;

pub struct Worker {
    pub id: String,
    pub name: String,
    map: HashMap<Vector3, Tile>,
    receiver: Receiver,
    pool: Arc<mysql::Pool>,
}

impl Worker {
    pub fn from_map(map: model::Map, db: Arc<mysql::Pool>, receiver: Receiver) -> Self {
        let id = map.id;

        let name = map.name;

        let inner = map
            .tiles
            .into_iter()
            .map(|(position, placable)| (position, Tile::from_placable_model(placable)))
            .collect();

        Worker {
            id,
            name,
            map: inner,
            receiver,
            pool: db,
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(result) = self.receiver.recv().await {
                println!("{result:?}");
            }
        }
    }
}
