use east_online_core::model::{Direction, Id};
use tokio::time;

pub struct Object {
    pub id: Id,
    pub movable: Option<Movable>,
}

pub struct Movable {
    pub direction: Direction,
    pub updated_at: Option<time::Instant>,
}
