use std::{collections::BinaryHeap, error::Error, io};
use tokio::net::{TcpListener, TcpStream};

use crate::{
    net::{io::Reader, packet},
    schedule::Schedule,
    selector::{ScheduleQueue, Waitings},
};

use super::job::Job;

pub struct Worker {
    listener: TcpListener,
    streams: Vec<TcpStream>,
    schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Worker {
    pub fn new(listener: TcpListener) -> Self {
        Worker {
            listener,
            streams: Vec::new(),
            schedule_queue: BinaryHeap::new(),
        }
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            if let Err(e) = self.handle_job(job) {
                eprintln!("{e}");
            }
        }
    }

    async fn select_job(&mut self) -> Job {
        if self.schedule_queue.is_first_urgent() {
            return self.schedule_queue.pop().unwrap().job;
        }

        tokio::select! {
            Ok((stream, _)) = self.listener.accept() => {
                Job::Accept(stream)
            }
            Ok(index) = self.streams.wait_for_readable() => {
                Job::Readable(index)
            }
            Ok(_) = self.schedule_queue.wait_for_first() => {
                self.schedule_queue.pop().unwrap().job
            },
        }
    }

    fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Accept(stream) => {
                println!("{:?} accepted", stream.peer_addr()?);

                self.streams.push(stream);

                Ok(())
            }
            Job::Drop(i) => {
                let stream = self.streams.remove(i);

                println!("{:?} dropped", stream.peer_addr()?);

                Ok(())
            }
            Job::Readable(i) => {
                let stream = self.streams.get(i).ok_or("stream not found")?;

                let schedule = match stream.try_read_packet() {
                    Ok(packet) => Schedule::instant(Job::Incoming(packet)),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                    Err(e) => {
                        eprintln!("{e}");

                        Schedule::instant(Job::Drop(i))
                    }
                };

                self.schedule_queue.push(schedule);

                Ok(())
            }
            Job::Incoming(packet) => self.handle_packet(packet),
        }
    }

    fn handle_packet(&mut self, packet: packet::Incoming) -> Result<(), Box<dyn Error>> {
        match packet {
            packet::Incoming::Hello { token } => {
                println!("{:?}", token);

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
