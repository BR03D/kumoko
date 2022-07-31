//! Module for Client functionality. Enable the client feature to use it.

#[cfg(feature = "bevy")]
pub mod bevy;

use std::{io, time::Duration};
use tokio::{net::{ToSocketAddrs, TcpStream}, sync::mpsc};
use crate::{Message, instance, event::{Origin, Event}};

pub use tokio::sync::mpsc::error::TryRecvError;

#[derive(Debug)]
/// A Client with a full duplex connection to a Server. Can be .into_split()
/// into an Emitter and Collector for async operations.
pub struct Client<Req: Message, Res: Message>{
    collector: Collector<Res>,
    emitter: Emitter<Req>,
}

impl<Req: Message, Res: Message> Client<Req, Res>{
    /// Connects to the server with the default Config.
    pub async fn connect<A: ToSocketAddrs>(ip: A) -> io::Result<Client<Req, Res>> {
        Self::connect_with_config(ip, Config::default()).await
    }

    /// Connects to the server with a custom Config.
    pub async fn connect_with_config<A: ToSocketAddrs>(ip: A, config: Config) -> io::Result<Client<Req, Res>> {
        let stream = TcpStream::connect(ip).await?;
        let (read, write) = stream.into_split();
    
        let (sx, rx) = mpsc::channel(config.emitter_buffer);
        instance::Collector::spawn_on_task(read, sx, Origin::OnClient, config.timeout);
        let collector = Collector{rx};
    
        let (sx, rx) = mpsc::channel(config.collector_buffer);
        instance::Emitter::spawn_on_task(write, rx);
        let emitter = Emitter{sx};
        
        Ok(Client{collector, emitter})
    }

    /// Gets the next event if one is available, otherwise it waits until it is.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_event(&mut self) -> Option<Event<Res>> {
        self.collector.get_event().await
    }

    /// Convenience method for applications which only care about requests.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_response(&mut self) -> Option<Res> {
        self.collector.get_response().await
    }

    pub fn try_get_event(&mut self) -> Result<Event<Res>, TryRecvError> {
        self.collector.try_get_event()
    }

    pub fn try_get_response(&mut self) -> Result<Res, TryRecvError> {
        self.collector.try_get_response()
    }

    /// Default method for streaming to the Server.
    pub async fn emit_request(&self, req: Req) {
        self.emitter.emit_request(req).await
    }

    pub fn try_emit(&self, req: Req) {
        self.emitter.try_emit(req)
    }

    /// Splits the Client into a Collector and a Emitter. The Emitter can be 
    /// cloned for async operations.
    pub fn into_split(self) -> (Collector<Res>, Emitter<Req>) {
        (self.collector, self.emitter)
    }
}

#[derive(Debug)]
pub struct Collector<Res: Message>{
    rx: mpsc::Receiver<(Event<Res>, Origin)>
}

impl<Res: Message> Collector<Res> {
    /// Gets the next event if one is available, otherwise it waits until it is.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_event(&mut self) -> Option<Event<Res>> {
        match self.rx.recv().await {
            Some((msg, _)) => Some(msg),
            None => None,
        }
    }

    /// Convenience method for applications which only care about requests.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_response(&mut self) -> Option<Res> {
        loop{
            match self.get_event().await{
                Some(Event::Message(res)) => return Some(res),
                Some(_) => continue,
                None => return None,
            }
        }
    }

    pub fn try_get_event(&mut self) -> Result<Event<Res>, TryRecvError> {
        match self.rx.try_recv() {
            Ok((msg, _)) => Ok(msg),
            Err(e) => Err(e),
        }
    }

    pub fn try_get_response(&mut self) -> Result<Res, TryRecvError> {
        loop{
            match self.try_get_event(){
                Ok(Event::Message(res)) => return Ok(res),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Emitter<Req: Message>{
    sx: mpsc::Sender<Req>
}

impl<Req: Message> Emitter<Req> {
    /// Default method for streaming to the Server.
    pub async fn emit_request(&self, req: Req) {
        match self.sx.send(req).await {
            Ok(_) => (),
            Err(_) => unreachable!(),
        }
    }

    pub fn try_emit(&self, req: Req) {
        match self.sx.try_send(req){
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    }
}

/// Config for the Client
pub struct Config{
    /// If no new Responses appear within this duration, we drop the collector.
    pub timeout: Duration,
    /// The size of the channel buffer for the Emitter.
    pub emitter_buffer: usize,
    /// the size of the channel buffer for the Collector.
    pub collector_buffer: usize,
}

impl Default for Config{
    fn default() -> Config {
        Config { timeout: Duration::MAX, emitter_buffer: 3, collector_buffer: 3 }
    }
}