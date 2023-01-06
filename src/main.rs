use std::{error::Error, sync::Arc};

use east_online_core::model;
use east_online_server::{
    db::DB,
    env::{self, url, CDN_ORIGIN},
    gate, map,
};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env::init();

    let pool = Arc::new(mysql::Pool::init()?);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let mut gate_worker = gate::Worker::new(pool.clone(), listener);

    let map_manifest = fetch_map_manifest().await?;

    for item in map_manifest.items {
        let map = fetch_map(&item.id).await?;

        let (tx, rx) = mpsc::channel(16);

        gate_worker.add_sender(&map.id, tx);

        let map_worker = map::Worker::from_map(map, pool.clone(), rx);

        tokio::spawn(async move {
            let id = map_worker.id.clone();

            if let Err(e) = map_worker.run().await {
                eprintln!("{} worker died for {e}", id);
            }
        });
    }

    println!("open gate");

    if let Err(e) = gate_worker.run().await {
        eprintln!("gate worker died for {e}");
    }

    Ok(())
}

async fn fetch_map_manifest() -> Result<model::MapManifest, Box<dyn Error>> {
    let response = reqwest::get(url(CDN_ORIGIN, "maps/manifest.yml")).await?;

    let bytes = response.bytes().await?;

    let result: model::MapManifest = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}

async fn fetch_map(id: &str) -> Result<model::Map, Box<dyn Error>> {
    let response = reqwest::get(url(CDN_ORIGIN, &format!("maps/{}.yml", id))).await?;

    let bytes = response.bytes().await?;

    let result: model::Map = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}
