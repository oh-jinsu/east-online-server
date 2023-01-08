use tokio::time;

use east_online_core::model::Direction;

pub struct Person {
    pub id: String,
    pub movable: Movable,
}

impl Person {
    pub fn new(id: String) -> Self {
        Person {
            id,
            movable: Movable::new(),
        }
    }
}

pub struct Movable {
    pub direction: Direction,
    pub updated_at: Option<time::Instant>,
}

impl Movable {
    pub fn new() -> Self {
        Movable {
            direction: Direction::Idle,
            updated_at: None,
        }
    }
}
