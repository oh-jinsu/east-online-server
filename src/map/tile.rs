use east_online_core::model::{self, Rotation};

use super::object::Object;

pub struct Tile {
    pub rotation: Rotation,
    pub object: Option<Object>,
}

impl Tile {
    pub fn from_placable_model(placable: model::Placable) -> Self {
        Tile {
            rotation: placable.rotation,
            object: None,
        }
    }
}
