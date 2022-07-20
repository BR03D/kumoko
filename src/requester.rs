use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{events::Request, client_requester::ClientRequester};

pub struct Requester{
    server_sender: mpsc::Sender<Request>,
    sender: mpsc::Sender<RequesterMessage>,
    reciever: mpsc::Receiver<RequesterMessage>,
}

impl Requester {
    pub fn spawn_on_task() -> 
    (mpsc::Sender<RequesterMessage>, mpsc::Receiver<Request>)
    {
        let (self_sender, self_reciever) = mpsc::channel(32);
        let (server_sender, server_reciever) = mpsc::channel(32);


        let r = Requester {
            reciever: self_reciever,
            sender: self_sender.clone(),
            server_sender,
        };
        tokio::spawn(async move{
            r.recv_loop().await;
        });

        (self_sender, server_reciever)
    }
    
    async fn recv_loop (mut self) {
        loop{
            let msg = match self.reciever.recv().await {
                Some(msg) => msg,
                None => return,
            };

            match msg {
                RequesterMessage::AddConnection(stream, idx) => {
                    ClientRequester::spawn_on_task(
                        stream, self.sender.clone(), idx
                    );
                },
                RequesterMessage::Incoming(req, _idx) => 
                self.server_sender.send(req).await.unwrap(),
            };
            tokio::task::yield_now().await;
        }
    }
}

#[derive(Debug)]
pub enum RequesterMessage {
    AddConnection(OwnedReadHalf, usize),
    Incoming(Request, usize),
}

