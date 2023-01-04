use std::error::Error;
use tokio::net::TcpListener;

use super::job::Job;

pub struct Keeper {
    listener: TcpListener,
}

impl Keeper {
    pub fn new(listener: TcpListener) -> Self {
        Keeper { listener }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            self.handle_job(job)?;
        }
    }

    async fn select_job(&mut self) -> Job {
        tokio::select! {
            Ok((stream, _)) = self.listener.accept() => {
                Job::Accept(stream)
            }
        }
    }

    fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Accept(_) => todo!(),
        }
    }
}
