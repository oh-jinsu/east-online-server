use std::error::Error;

use east_online_core::extension::ByteArray;

#[derive(Debug)]
pub enum Incoming {
    Ping { timestamp: i64 },
}

impl Incoming {
    pub fn deserialize(buf: &[u8]) -> Result<Self, Box<dyn Error>> {
        if buf.len() < 2 {
            return Err(format!("buffer too short to deserialize, {buf:?}").into());
        }

        let serial = u16::from_le_bytes([buf[0], buf[1]]);

        let body = &buf[2..];

        match serial {
            1 => {
                if body.len() < 8 {
                    return Err(format!("buffer too short to deserialize, {buf:?}").into());
                }

                Ok(Self::Ping {
                    timestamp: i64::from_le_bytes(body.clone_into_array()),
                })
            }
            n => Err(format!("unexpected packet arrived, {n:?}").into()),
        }
    }
}
