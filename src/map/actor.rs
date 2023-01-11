use tokio::time;

use east_online_core::model::Direction;

pub struct Actor {
    pub id: String,
    pub movable: Movable,
}

impl Actor {
    pub fn new(id: String) -> Self {
        Actor {
            id,
            movable: Movable::new(),
        }
    }
}

pub struct Movable {
    pub direction: Direction,
    pub moved_at: time::Instant,
}

impl Movable {
    pub fn new() -> Self {
        Movable {
            direction: Direction::Idle,
            moved_at: time::Instant::now(),
        }
    }
}
