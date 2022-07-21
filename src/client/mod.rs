use std::io;

use tokio::{net::{ToSocketAddrs, TcpStream}, sync::mpsc};

use crate::{Message, Origin, instance};

pub async fn connect<A: ToSocketAddrs, Req: Message, Res: Message>(
    ip: A
) -> io::Result<(Receiver<Req>, Sender<Res>)> {

    let stream = TcpStream::connect(ip).await?;
    let (read, write) = stream.into_split();

    let (sx, rx) = mpsc::channel(32);
    instance::Receiver::spawn_on_task(read, sx, Origin::OnClient);
    let receiver = Receiver{rx};

    let (sx, rx) = mpsc::channel(32);
    instance::Sender::spawn_on_task(write, rx);
    let sender = Sender{sx};
    
    Ok((receiver, sender))
}


#[derive(Debug)]
pub struct Receiver<Res>{
    rx: mpsc::Receiver<(Res, Origin)>
}

impl<Res: Message> Receiver<Res> {
    pub async fn get_response(&mut self) -> Res {
        match self.rx.recv().await {
            Some((msg, _)) => msg,
            None => panic!("WE CRASHED"),
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