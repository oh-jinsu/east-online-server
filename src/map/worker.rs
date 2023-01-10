use east_online_core::model::{self, Vector3};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{
    map::Person,
    net::{
        io::{get_packet_buf, Reader},
        packet,
    },
    schedule::Schedule,
    selector::{ScheduleQueue, Waitings},
};

use super::{Job, Tile};
use std::{
    collections::{BinaryHeap, HashMap},
    error::Error,
    io,
    sync::Arc,
};

type Sender = mpsc::Sender<(TcpStream, String, Vector3)>;

type Receiver = mpsc::Receiver<(TcpStream, String, Vector3)>;

pub struct Worker {
    id: String,
    name: String,
    map: HashMap<Vector3, Tile>,
    channel: (Sender, Receiver),
    pool: Arc<mysql::Pool>,
    streams: HashMap<String, (TcpStream, Vector3)>,
    schedule_queue: BinaryHeap<Schedule<Job>>,
}

impl Worker {
    pub fn from_map(map: model::Map, db: Arc<mysql::Pool>, channel: (Sender, Receiver)) -> Self {
        Worker {
            id: map.id,
            name: map.name,
            map: map
                .tiles
                .into_iter()
                .map(|(position, placable)| (position, Tile::from_placable(placable)))
                .collect(),
            channel,
            pool: db,
            streams: HashMap::new(),
            schedule_queue: ScheduleQueue::new(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
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
            Some((stream, id, position)) = self.channel.1.recv() => {
                Job::Accept(stream, id, position)
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
     * Throw an error if something went wrong with itself.
     */
    async fn handle_job(&mut self, job: Job) -> Result<(), Box<dyn Error>> {
        match job {
            Job::Accept(stream, id, position) => {
                if let Some(tile) = self.map.get_mut(&position) {
                    println!("{:?} accepted by {}", stream.peer_addr()?, self.id);

                    let person = Person::new(id.to_owned());

                    tile.people.insert(id.to_owned(), person);

                    self.streams.insert(id.clone(), (stream, position));

                    let users = self
                        .streams
                        .iter()
                        .map(|(key, (_, position))| (key.to_owned(), position.to_owned()))
                        .collect();

                    let packet = packet::Outgoing::Hello {
                        map_id: self.id.to_owned(),
                        users,
                    };

                    let schedule = Schedule::instant(Job::Write(id, packet));

                    self.schedule_queue.push(schedule);

                    Ok(())
                } else {
                    Err("wrong position".into())
                }
            }
            Job::Drop(key, reason) => {
                if let Some((stream, position)) = self.streams.remove(&key) {
                    if let Some(tile) = self.map.get_mut(&position) {
                        tile.people.remove(&key);
                    }

                    let addr = stream.peer_addr()?;

                    println!("{:?} dropped for {}", addr, reason);

                    Ok(())
                } else {
                    Err("drop failed".into())
                }
            }
            Job::Readable(key) => {
                let (stream, _) = self.streams.get(&key).ok_or("stream not found")?;

                let schedule = match stream.try_read_packet() {
                    Ok(packet) => Schedule::instant(Job::Incoming(key, packet)),
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                    Err(e) => Schedule::instant(Job::Drop(key, format!("{e}"))),
                };

                self.schedule_queue.push(schedule);

                Ok(())
            }
            Job::Incoming(key, packet) => {
                if let Err(e) = self.handle_packet(key.to_owned(), packet).await {
                    let schedule = Schedule::instant(Job::Drop(key, format!("{e}")));

                    self.schedule_queue.push(schedule);
                }

                Ok(())
            }
            Job::Write(key, packet) => {
                if let Some((stream, _)) = self.streams.get(&key) {
                    let buf = get_packet_buf(packet)?;

                    match stream.try_write(&buf) {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            let job = Job::Drop(key.to_owned(), format!("{e}"));

                            let schedule = Schedule::instant(job);

                            self.schedule_queue.push(schedule);
                        }
                    }
                }

                Ok(())
            }
            Job::Broadcast(packet) => {
                let buf = get_packet_buf(packet)?;

                for (key, (stream, _)) in &self.streams {
                    match stream.try_write(&buf) {
                        Ok(_) => {
                            continue;
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            let job = Job::Drop(key.to_owned(), format!("{e}"));

                            let schedule = Schedule::instant(job);

                            self.schedule_queue.push(schedule);
                        }
                    }
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
        key: String,
        packet: packet::Incoming,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
