use std::collections::HashMap;

use tokio::{net::tcp::OwnedWriteHalf, sync::{mpsc, oneshot}};

use crate::{events::Response, MyError};

#[derive(Debug)]
pub struct Responder {
    map: HashMap<usize, OwnedWriteHalf>,
    int: usize,
    rx: mpsc::Receiver<ResponderMessage>,
}

impl Responder {
    pub fn new() -> mpsc::Sender<ResponderMessage>{
        let (sx, rx) = mpsc::channel(32);

        let r = Responder { map: HashMap::new(), int:0, rx };
        tokio::spawn(async move{
            r.recv_loop().await;
            println!("ERROR HERE");
        });

        sx
    }

    async fn recv_loop (mut self) {
        loop{
            let msg = match self.rx.recv().await {
                Some(msg) => msg,
                None => return,
            };

            match msg {
                ResponderMessage::AddConnection(owh, sx) => {
                    sx.send(self.add_connection(owh)).unwrap();
                },
                ResponderMessage::SendResponse(resp, index) => {
                    self.respond(self.map.get(&index).unwrap(), &resp).await.unwrap();
                },
            };
            tokio::task::yield_now().await;
        }
    }

    pub async fn respond(&self, stream: &OwnedWriteHalf, resp: &Response) -> Result<(), MyError> {
        stream.writable().await?;
        stream.try_write(&bincode::serialize(resp)?)?;
    
        Ok(())
    }

    fn add_connection(&mut self, owh: OwnedWriteHalf) -> usize {
        self.int += 1;
        self.map.insert(self.int, owh);

        self.int
    }
}

#[derive(Debug)]
pub enum ResponderMessage {
    AddConnection(OwnedWriteHalf, oneshot::Sender<usize>),
    SendResponse(Response, usize),
}