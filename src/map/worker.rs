use east_online_core::model::{self, Direction, Vector3};
use tokio::{net::TcpStream, sync::mpsc, time};

use crate::{
    map::Actor,
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

                    let person = Actor::new(id.to_owned());

                    tile.actors.insert(id.to_owned(), person);

                    self.streams.insert(id.clone(), (stream, position));

                    let users = self
                        .streams
                        .iter()
                        .map(|(key, (_, position))| (key.to_owned(), position.to_owned()))
                        .collect();

                    let packet = packet::Outgoing::Hello {
                        id: id.to_owned(),
                        map_id: self.id.to_owned(),
                        actors: users,
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
                        tile.actors.remove(&key);
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
            Job::Move(key, duration) => {
                let (_, position) = self.streams.get_mut(&key).ok_or("no stream")?;

                let next = {
                    let current_tile = self.map.get(position).ok_or("no tile")?;

                    let actor = current_tile.actors.get(&key).ok_or("no actor")?;

                    match actor.movable.direction {
                        Direction::Idle => {
                            let packet = packet::Outgoing::Stop {
                                id: key.to_owned(),
                                position: position.to_owned(),
                            };

                            let schedule = Schedule::instant(Job::Broadcast(packet));

                            self.schedule_queue.push(schedule);

                            return Ok(());
                        }
                        Direction::Up => Vector3 {
                            x: position.x,
                            y: position.y,
                            z: position.z + 1,
                        },
                        Direction::Right => Vector3 {
                            x: position.x + 1,
                            y: position.y,
                            z: position.z,
                        },
                        Direction::Down => Vector3 {
                            x: position.x,
                            y: position.y,
                            z: position.z - 1,
                        },
                        Direction::Left => Vector3 {
                            x: position.x - 1,
                            y: position.y,
                            z: position.z,
                        },
                    }
                };

                if let None = self.map.get(&next) {
                    let packet = packet::Outgoing::Stop {
                        id: key.to_owned(),
                        position: position.to_owned(),
                    };

                    let schedule = Schedule::instant(Job::Broadcast(packet));

                    self.schedule_queue.push(schedule);

                    return Ok(());
                }

                let mut actor = self
                    .map
                    .get_mut(position)
                    .ok_or("no actor")?
                    .actors
                    .remove(&key)
                    .ok_or("no actor")?;

                actor.movable.moved_at = time::Instant::now();

                self.map
                    .get_mut(&next)
                    .unwrap()
                    .actors
                    .insert(key.to_owned(), actor);

                *position = next.to_owned();

                let packet = packet::Outgoing::Move {
                    id: key.to_owned(),
                    position: next.to_owned(),
                    duration,
                };

                self.schedule_queue
                    .push(Schedule::instant(Job::Broadcast(packet)));

                let deadline = time::Instant::now() + duration;

                self.schedule_queue
                    .push(Schedule::new(Job::Move(key, duration), deadline));

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
        match packet {
            packet::Incoming::Move { direction } => {
                let (_, position) = self.streams.get(&key).ok_or("no stream")?;

                let tile = self.map.get_mut(position).ok_or("no tile")?;

                let actor = tile.actors.get_mut(&key).ok_or("no actor")?;

                if actor.movable.direction == direction {
                    return Ok(());
                }

                let duration = time::Duration::from_millis(300);

                let last_direction = actor.movable.direction;

                actor.movable.direction = direction;

                let is_cool = actor.movable.moved_at + duration > time::Instant::now();

                if is_cool || direction == Direction::Idle || last_direction != Direction::Idle {
                    return Ok(());
                }

                let job = Job::Move(key, duration);

                self.schedule_queue.push(Schedule::instant(job));

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
