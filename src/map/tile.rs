use std::collections::HashMap;

use east_online_core::model::{self, Rotation};

use super::{object::Object, Actor};

pub struct Tile {
    pub rotation: Rotation,
    pub object: Option<Object>,
    pub actors: HashMap<String, Actor>,
}

impl Tile {
    pub fn from_placable(placable: model::Placable) -> Self {
        Tile {
            rotation: placable.rotation,
            object: None,
            actors: HashMap::new(),
        }
    }
}
