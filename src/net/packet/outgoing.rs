#[derive(Debug)]
pub enum Outgoing {
    Pong { timestamp: i64 },
}

impl Outgoing {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Outgoing::Pong { timestamp } => {
                [&[1 as u8, 0] as &[u8], &timestamp.to_le_bytes()].concat()
            }
        }
    }
}
