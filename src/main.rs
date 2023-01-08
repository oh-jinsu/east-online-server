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

    println!("fetch manifest");

    let mut gate_worker = gate::Worker::new(pool.clone(), listener);

    let map_manifest = fetch_map_manifest().await?;

    for item in map_manifest.items {
        let map = fetch_map(&item.id).await?;

        let map_id = map.id.clone();

        let (enter_tx, enter_rx) = mpsc::channel(16);

        let (exit_tx, exit_rx) = mpsc::channel(16);

        gate_worker.add_channel(&map_id, (enter_tx, exit_rx));

        println!("create worker, {}", &map_id);

        let map_worker = map::Worker::from_map(map, pool.clone(), (exit_tx, enter_rx));

        tokio::spawn(async move {
            if let Err(e) = map_worker.run().await {
                eprintln!("{} worker died for {e}", map_id);
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
