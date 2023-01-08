use east_online_core::model::Vector3;

#[derive(Debug)]
pub enum Outgoing {
    Hello {
        map_id: String,
        position: Vector3,
        users: Vec<(String, Vector3)>,
    },
}

impl Outgoing {
    pub fn serialize(self) -> Vec<u8> {
        match self {
            Outgoing::Hello {
                map_id,
                position,
                users,
            } => {
                let users: Vec<u8> = users
                    .iter()
                    .flat_map(|(user_id, position)| {
                        [user_id.as_bytes(), &position.to_bytes()].concat()
                    })
                    .collect();

                [
                    &[1 as u8, 0] as &[u8],
                    map_id.as_bytes(),
                    &position.to_bytes(),
                    &users,
                ]
                .concat()
            }
        }
    }
}
