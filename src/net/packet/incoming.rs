use std::error::Error;

use east_online_core::model::Direction;

#[derive(Debug)]
pub enum Incoming {
    Hello { token: String },
    Move { direction: Direction },
}

impl Incoming {
    pub fn deserialize(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 2 {
            return Err(format!("buffer too short to deserialize, {buf:?}").into());
        }

        let serial = u16::from_le_bytes([buf[0], buf[1]]);

        let body = &buf[2..];

        match serial {
            1 => Ok(Self::Hello {
                token: String::from_utf8_lossy(body).to_string(),
            }),
            2 => {
                let direction = match &body[0] {
                    0 => Direction::Idle,
                    1 => Direction::Up,
                    2 => Direction::Right,
                    3 => Direction::Down,
                    4 => Direction::Left,
                    _ => return Err("unknown direction".into())
                };

                Ok(Self::Move {
                    direction
                })
            },
            n => Err(format!("unexpected packet arrived, {n:?}").into()),
        }
    }
}
