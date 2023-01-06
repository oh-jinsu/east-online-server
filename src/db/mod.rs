use std::error::Error;

pub trait DB {
    fn init() -> Result<mysql::Pool, Box<dyn Error>>;
}

impl DB for mysql::Pool {
    fn init() -> Result<Self, Box<dyn Error>> {
        let url = std::env::var("DATABASE_URL")?;

        let opts = mysql::Opts::from_url(&url)?;

        let pool = mysql::Pool::new(opts)?;

        Ok(pool)
    }
}
