use east_online_core::model::{self, Vector3};
use mysql::{params, prelude::*};
use reqwest::{header::AUTHORIZATION, StatusCode};
use std::{
    collections::{BinaryHeap, HashMap},
    error::Error,
    io,
    sync::Arc,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::{
    env::{url, API_ORIGIN},
    net::{io::Reader, packet},
    schedule::Schedule,
    selector::{ScheduleQueue, Waitings},
};

use super::job::Job;

type Receiver = mpsc::Receiver<(TcpStream, String, Vector3)>;

type Sender = mpsc::Sender<(TcpStream, String, Vector3)>;

pub struct Worker {
    listener: TcpListener,
    streams: Vec<TcpStream>,
    schedule_queue: BinaryHeap<Schedule<Job>>,
    db: Arc<mysql::Pool>,
    channels: HashMap<String, (Sender, Receiver)>,
}

impl Worker {
    pub fn new(db: Arc<mysql::Pool>, listener: TcpListener) -> Self {
        Worker {
            listener,
            streams: Vec::new(),
            schedule_queue: BinaryHeap::new(),
            db,
            channels: HashMap::new(),
        }
    }

    pub fn add_channel(&mut self, key: &str, channel: (Sender, Receiver)) {
        self.channels.insert(key.to_string(), channel);
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        loop {
            let job = self.select_job().await;

            if let Err(e) = self.handle_job(job).await {
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

    /**
     * Handle a scheduled job.
     *
     * Throw an error if something goes wrong with itself.
     */
    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Accept(stream) => {
                println!("{:?} accepted by gate", stream.peer_addr()?);

                self.streams.push(stream);

                Ok(())
            }
            Job::Drop(index, reason) => {
                let stream = self.streams.remove(index);

                println!("{:?} dropped for {}", stream.peer_addr()?, reason);

                Ok(())
            }
            Job::Readable(index) => {
                let stream = self.streams.get(index).ok_or("stream not found")?;

                let schedule = match stream.try_read_packet() {
                    Ok(packet) => Schedule::instant(Job::Incoming(index, packet)),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                    Err(e) => Schedule::instant(Job::Drop(index, format!("{e}"))),
                };

                self.schedule_queue.push(schedule);

                Ok(())
            }
            Job::Incoming(index, packet) => {
                if let Err(e) = self.handle_packet(index, packet).await {
                    let schedule = Schedule::instant(Job::Drop(index, format!("{e}")));

                    self.schedule_queue.push(schedule);
                }

                Ok(())
            }
            Job::Send {
                index,
                user_id,
                map_id,
            } => {
                let stream = self.streams.remove(index);

                if let Some((sender, _)) = self.channels.get(&map_id) {
                    sender
                        .send((stream, user_id, Vector3 { x: 0, y: 0, z: 0 }))
                        .await?;
                }

                Ok(())
            }
        }
    }

    /**
     * Handle a incoming packet from a stream.
     *
     * Throw an error if something went wrong with the stream.
     * The stream is going to be dropped immediately.
     */
    async fn handle_packet(
        &mut self,
        index: usize,
        packet: packet::Incoming,
    ) -> Result<(), Box<dyn Error>> {
        match packet {
            packet::Incoming::Hello { token } => {
                let response = reqwest::Client::new()
                    .get(url(API_ORIGIN, "auth"))
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .send()
                    .await?;

                match response.status() {
                    StatusCode::CREATED => {
                        let token = response.json::<model::Token>().await?;

                        let mut conn = self.db.get_conn()?;

                        let user_id: String = match conn
                            .exec_first(
                                "SELECT id FROM users WHERE id = :id",
                                mysql::params! { "id" => token.id },
                            )?
                            .map(|id| id)
                        {
                            Some(user_id) => user_id,
                            None => return Err("user not found".into()),
                        };

                        let map_id: String = conn
                            .exec_first(
                                "SELECT map_id FROM locations WHERE id = :id",
                                mysql::params! { "id" => user_id.clone() },
                            )?
                            .map(|map_id| map_id)
                            .unwrap_or(String::from("map_0000"));

                        let job = Job::Send {
                            index,
                            user_id,
                            map_id,
                        };

                        let schedule = Schedule::instant(job);

                        self.schedule_queue.push(schedule);

                        Ok(())
                    }
                    _ => Err(response.text().await?.into()),
                }
            }
            _ => Ok(()),
        }
    }
}
