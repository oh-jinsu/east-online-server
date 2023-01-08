use std::collections::HashMap;

use east_online_core::model::{self, Rotation};

use super::{object::Object, Person};

pub struct Tile {
    pub rotation: Rotation,
    pub object: Option<Object>,
    pub people: HashMap<String, Person>,
}

impl Tile {
    pub fn from_placable(placable: model::Placable) -> Self {
        Tile {
            rotation: placable.rotation,
            object: None,
            people: HashMap::new(),
        }
    }
}
