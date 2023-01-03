use std::error::Error;

use east_online_core::data::{Map, MapManifest};
use east_online_server::{env::get_cdn_origin, Gatekeeper};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let gatekeeper = Gatekeeper::new(listener);

    let map_manifest = fetch_map_manifest().await?;

    for item in map_manifest.items {
        let map = fetch_map(&item.id).await?;
    }

    gatekeeper.keep().await
}

async fn fetch_map_manifest() -> Result<MapManifest, Box<dyn Error>> {
    let response = reqwest::get(format!("{}/maps/manifest.yml", get_cdn_origin())).await?;

    let bytes = response.bytes().await?;

    let result: MapManifest = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}

async fn fetch_map(id: &str) -> Result<Map, Box<dyn Error>> {
    let response = reqwest::get(format!("{}/maps/{}.yml", get_cdn_origin(), id)).await?;

    let bytes = response.bytes().await?;

    let result: Map = serde_yaml::from_slice(&bytes)?;

    Ok(result)
}
