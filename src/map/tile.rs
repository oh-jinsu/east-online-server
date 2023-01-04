use east_online_core::{data, models::Rotation};

use super::object::Object;

pub struct Tile {
    pub rotation: Rotation,
    pub object: Option<Object>,
}

impl Tile {
    pub fn from_placable(data: data::Placable) -> Self {
        Tile {
            rotation: data.rotation,
            object: None,
        }
    }
}
