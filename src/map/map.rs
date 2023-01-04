use std::collections::HashMap;

use east_online_core::model::{self, Vector3};

use super::tile::Tile;

pub struct Map {
    inner: HashMap<Vector3, Tile>,
}

impl Map {
    pub fn from_model(map: model::Map) -> Self {
        let inner = map
            .tiles
            .into_iter()
            .map(|(position, placable)| (position, Tile::from_placable_model(placable)))
            .collect();

        Map { inner }
    }
}
