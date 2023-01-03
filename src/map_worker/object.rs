use east_online_core::models::{Direction, Id};
use tokio::time;

pub struct Object {
    pub id: Id,
}

pub struct Movable {
    pub direction: Direction,
    pub updated_at: Option<time::Instant>,
}
