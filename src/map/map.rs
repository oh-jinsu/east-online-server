use std::collections::HashMap;

use east_online_core::{data, models::Vector3};

use super::tile::Tile;

pub struct Map {
    inner: HashMap<Vector3, Tile>,
}

impl Map {
    pub fn from_data(data: data::Map) -> Self {
        let inner = data
            .tiles
            .into_iter()
            .map(|(position, placable)| (position, Tile::from_placable(placable)))
            .collect();

        Map { inner }
    }
}
