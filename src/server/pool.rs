use std::collections::HashMap;

use tokio::{sync::mpsc, net::tcp::OwnedWriteHalf};

use crate::{Message, server::Target, instance};


///Lives on a seperate task
/// 
/// Handles adding and removing Senders and propagating Messages.
pub struct SenderPool<Res>{
    map: HashMap<usize, mpsc::Sender<Res>>,
    rx: mpsc::Receiver<PoolMessage<Res>>
}

impl<Res: Message> SenderPool<Res> {
    pub fn spawn_on_task() -> mpsc::Sender<PoolMessage<Res>> {
        let (sx, rx) = mpsc::channel(32);
        SenderPool{ rx, map: HashMap::new(), }.recv_loop();

        sx
    }

    fn recv_loop(mut self) {
        tokio::spawn(async move{
            loop{
                let msg = self.rx.recv().await.unwrap();
                self.handle_msg(msg).await;
                tokio::task::yield_now().await;
            }
        });
    }

    async fn handle_msg(&mut self, msg: PoolMessage<Res>) {
        match msg {
            PoolMessage::Join(stream, id) => {
                let (sx, rx) = mpsc::channel(32);
                instance::Sender::spawn_on_task(stream, rx);
                self.map.insert(id, sx);
            },
            PoolMessage::Msg(msg) => self.send(msg).await,
            PoolMessage::Leave(id) => {self.map.remove(&id);},
        }
    }

    async fn send(&mut self, (res, target): (Res, Target)) {
        match target {
            Target::All => {
                for (_, sender) in self.map.iter() {
                    let res = res.clone();
                    sender.send(res).await.unwrap();
                }
            },
            Target::One(id) => 
                if let Some(sender) = self.map.get(&id) {
                    sender.send(res).await.unwrap();
                },
        }
    }
}

#[derive(Debug)]
pub enum PoolMessage<Msg>{
    Join(OwnedWriteHalf, usize),
    Msg((Msg, Target)),
    Leave(usize),
}