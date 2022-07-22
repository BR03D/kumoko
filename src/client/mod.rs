use std::io;

use tokio::{net::{ToSocketAddrs, TcpStream}, sync::mpsc};

use crate::{Message, Origin, instance, Event};

pub async fn connect<A: ToSocketAddrs, Req: Message, Res: Message>(
    ip: A
) -> io::Result<Client<Req, Res>> {
    let stream = TcpStream::connect(ip).await?;
    let (read, write) = stream.into_split();

    let (sx, rx) = mpsc::channel(32);
    instance::Receiver::spawn_on_task(read, sx, Origin::OnClient);
    let receiver = Receiver{rx};

    let (sx, rx) = mpsc::channel(32);
    instance::Sender::spawn_on_task(write, rx);
    let sender = Sender{sx};
    
    Ok(Client{receiver, sender})
}

#[derive(Debug)]
pub struct Client<Req, Res: Message>{
    receiver: Receiver<Res>,
    sender: Sender<Req>,
}

impl<Req: Message, Res: Message> Client<Req, Res>{
    pub async fn get_event(&mut self) -> Option<Event<Res>> {
        self.receiver.get_event().await
    }

    pub async fn get_response(&mut self) -> Option<Res> {
        self.receiver.get_response().await
    }

    pub async fn send_request(&self, req: Req) {
        self.sender.send_request(req).await
    }

    pub fn into_split(self) -> (Receiver<Res>, Sender<Req>) {
        (self.receiver, self.sender)
    }
}

#[derive(Debug)]
pub struct Receiver<Res: Message>{
    rx: mpsc::Receiver<(Event<Res>, Origin)>
}

impl<Res: Message> Receiver<Res> {
    pub async fn get_response(&mut self) -> Option<Res> {
        loop{
            match self.get_event().await{
                Some(e) => match e {
                    Event::Message(res) => return Some(res),
                    _ => continue
                },
                None => return None,
            }
        }
    }

    pub async fn get_event(&mut self) -> Option<Event<Res>> {
        match self.rx.recv().await {
            Some((msg, _)) => Some(msg),
            None => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sender<Req>{
    sx: mpsc::Sender<Req>
}

impl<Req: Message> Sender<Req> {
    pub async fn send_request(&self, req: Req) {
        self.sx.send(req).await.unwrap();
    }
}