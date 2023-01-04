use std::error::Error;

use east_online_core::data;
use east_online_server::{
    env::get_cdn_origin,
    gate::Keeper,
    map::{Map, Worker},
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let keeper = Keeper::new(listener);

    let map_manifest = fetch_map_manifest().await?;

    for item in map_manifest.items {
        let map = fetch_map(&item.id).await?;

        let map = Map::from_data(map);

        let worker = Worker::new(map);

        tokio::spawn(async move {
            let _ = worker.run().await;
        });
    }

    keeper.run().await
}

async fn fetch_map_manifest() -> Result<data::MapManifest, Box<dyn Error>> {
    let response = reqwest::get(format!("{}/maps/manifest.yml", get_cdn_origin())).await?;

    let bytes = response.bytes().await?;

    let result: data::MapManifest = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}

async fn fetch_map(id: &str) -> Result<data::Map, Box<dyn Error>> {
    let response = reqwest::get(format!("{}/maps/{}.yml", get_cdn_origin(), id)).await?;

    let bytes = response.bytes().await?;

    let result: data::Map = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}
