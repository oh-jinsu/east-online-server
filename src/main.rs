use std::error::Error;

use east_online_core::model;
use east_online_server::{
    env::{init, url, CDN_ORIGIN},
    gate,
    map::{self, Map},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let gate_worker = gate::Worker::new(listener);

    let map_manifest = fetch_map_manifest().await?;

    for item in map_manifest.items {
        let map = fetch_map(&item.id).await?;

        let map = Map::from_model(map);

        let map_worker = map::Worker::new(map);

        tokio::spawn(async {
            let _ = map_worker.run().await;
        });
    }

    gate_worker.run().await
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
