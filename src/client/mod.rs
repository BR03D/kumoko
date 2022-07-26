use std::{io, time::Duration};

use tokio::{net::{ToSocketAddrs, TcpStream}, sync::mpsc};

use crate::{Message, instance, event::{Origin, Event}};

#[derive(Debug)]
/// A Client with a full duplex connection to a Server. Can be .into_split()
/// into a Reciever and Sender for async operations.
pub struct Client<Req: Message, Res: Message>{
    receiver: Receiver<Res>,
    sender: Sender<Req>,
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
    
        let (sx, rx) = mpsc::channel(config.sender_buffer);
        instance::Receiver::spawn_on_task(read, sx, Origin::OnClient, config.timeout);
        let receiver = Receiver{rx};
    
        let (sx, rx) = mpsc::channel(config.receiver_buffer);
        instance::Sender::spawn_on_task(write, rx);
        let sender = Sender{sx};
        
        Ok(Client{receiver, sender})
    }

    /// Gets the next event if one is available, otherwise it waits until it is.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_event(&mut self) -> Option<Event<Res>> {
        self.receiver.get_event().await
    }

    
    /// Convenience method for applications which only care about requests.
    /// 
    /// Will return `None` once the connection has ended.
    pub async fn get_response(&mut self) -> Option<Res> {
        self.receiver.get_response().await
    }

    /// Default method for streaming to the Server.
    pub async fn send_request(&self, req: Req) {
        self.sender.send_request(req).await
    }

    /// Splits the Client into a Receiver and a Sender. The Sender can be 
    /// cloned for async operations.
    pub fn into_split(self) -> (Receiver<Res>, Sender<Req>) {
        (self.receiver, self.sender)
    }
}

#[derive(Debug)]
pub struct Receiver<Res: Message>{
    rx: mpsc::Receiver<(Event<Res>, Origin)>
}

impl<Res: Message> Receiver<Res> {
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
}

#[derive(Debug, Clone)]
pub struct Sender<Req: Message>{
    sx: mpsc::Sender<Req>
}

impl<Req: Message> Sender<Req> {
    /// Default method for streaming to the Server.
    pub async fn send_request(&self, req: Req) {
        match self.sx.send(req).await {
            Ok(_) =>(), 
            Err(_) => unreachable!(),
        }
    }
}

/// Config for the Client
pub struct Config{
    /// If no new Responses appear within this duration, we drop the receiver.
    timeout: Duration,
    /// The size of the channel buffer for the Sender.
    sender_buffer: usize,
    /// the size of the channel buffer for the Receiver.
    receiver_buffer: usize,
}

impl Default for Config{
    fn default() -> Config {
        Config { timeout: Duration::MAX, sender_buffer: 3, receiver_buffer: 3 }
    }
}