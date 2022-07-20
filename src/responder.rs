use std::{collections::HashMap, io::ErrorKind};

use tokio::{net::tcp::OwnedWriteHalf, sync::mpsc};

use crate::{events::Response, MyError, server::{IResponse, Target}};


#[derive(Debug)]
pub struct Responder {
    map: HashMap<usize, mpsc::Sender<Response>>,
    rx: mpsc::Receiver<ResponderMessage>,
}

impl Responder {
    pub fn spawn_on_task() -> mpsc::Sender<ResponderMessage>{
        let (sx, rx) = mpsc::channel(32);

        let r = Responder { map: HashMap::new(),  rx };
        tokio::spawn(async move{
            r.recv_loop().await;
        });

        sx
    }
    
    async fn recv_loop (mut self) {
        loop{
            match self.rx.recv().await {
                Some(msg) => self.handle_msg(msg).await,
                None => panic!("Global Responder Crash"),
            };
            tokio::task::yield_now().await;
        }
    }

    async fn handle_msg(&mut self, msg: ResponderMessage) {
        match msg {
            ResponderMessage::AddConnection(stream, idx) => {
                let client = ClientResponder::spawn_on_task(stream);
                self.map.insert(idx, client);
            },
            ResponderMessage::SendResponse(res) => {
                self.send(res).await;
            },
        };
    }

    async fn send(&mut self, res: IResponse) {
        let msg = res.msg;
        match res.target {
            Target::All => {

                //very jank nononon
                //will delete entries on a send failure
                //send failure occurs only after the second failed attempt
                self.map.retain(|_idx, client| {
                    if let Err(_) = client.try_send(msg.clone()) { false }
                    else { true }
                });
            },
            Target::One(idx) => {
                if let Some(client) = self.map.get(&idx) {
                    if let Err(_) = client.send(msg).await{
                        self.map.remove(&idx);
                    };
                }
            },
        };
    }

}

pub struct ClientResponder{
    stream: OwnedWriteHalf,
    rx: mpsc::Receiver<Response>,
}

impl ClientResponder {
    fn spawn_on_task(stream: OwnedWriteHalf) -> mpsc::Sender<Response> {
        let (sx, rx) = mpsc::channel(32);
        let client = ClientResponder{ stream, rx };

        tokio::spawn(async move{
            client.respond_loop().await
        });

        sx
    }

    async fn respond_loop(mut self) {
        loop{
            match self.rx.recv().await {
                None => panic!("Client Responder Crash"),
                Some(res) => {
                    let err = self.respond(res).await;

                    if let Err(MyError::Io(e)) = &err{
                        match e.kind() {
                            ErrorKind::BrokenPipe => return,
                            ErrorKind::WouldBlock => continue,
                            _ => err.unwrap(),
                        }
                    }
                },
            }
            tokio::task::yield_now().await;
        }
    }

    async fn respond(&self, res: Response) -> Result<(), MyError> {
        self.stream.writable().await?;
        self.stream.try_write(&bincode::serialize(&res)?)?;
    
        Ok(())
    }
}

#[derive(Debug)]
pub enum ResponderMessage {
    AddConnection(OwnedWriteHalf, usize),
    SendResponse(IResponse),
}