use std::error::Error;

use tokio::time;

use east_online_core::model::Vector3;

#[derive(Debug)]
pub enum Outgoing {
    Hello {
        id: String,
        map_id: String,
        actors: Vec<(String, Vector3)>,
    },
    Move {
        id: String,
        position: Vector3,
        duration: time::Duration,
    },
    Stop {
        id: String,
        position: Vector3,
    }
}

impl Outgoing {
    pub fn serialize(self) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            Outgoing::Hello {
                id,
                map_id,
                actors,
            } => {
                let users: Vec<u8> = actors
                    .iter()
                    .flat_map(|(user_id, position)| {
                        [user_id.as_bytes(), &position.to_bytes()].concat()
                    })
                    .collect();

                Ok([
                    &[1 as u8, 0] as &[u8],
                    id.as_bytes(),
                    map_id.as_bytes(),
                    &users,
                ]
                .concat())
            }
            Outgoing::Move { id, position, duration } => Ok([
                &[2 as u8, 0] as &[u8],
                id.as_bytes(),
                &position.to_bytes(),
                &i64::try_from(duration.as_millis())?.to_le_bytes(),
            ].concat()),
            Outgoing::Stop { id, position } => Ok([
                &[3 as u8, 0] as &[u8],
                id.as_bytes(),
                &position.to_bytes(),
            ].concat()),
        }
    }
}
