use std::collections::HashMap;

use tokio::{sync::mpsc, net::tcp::OwnedWriteHalf};

use crate::{Message, server::Target, instance};


///Lives on a seperate task
/// 
/// Handles adding and removing Emitters and propagating Messages.
pub(crate) struct EmitterPool<Res>{
    map: HashMap<usize, mpsc::Sender<Res>>,
    rx: mpsc::Receiver<PoolMessage<Res>>,
    client_buffer: usize,
}

impl<Res: Message> EmitterPool<Res> {
    pub(crate) fn spawn_on_task(
        pool_buffer: usize,
        client_buffer: usize,
    ) -> mpsc::Sender<PoolMessage<Res>> {
        let (sx, rx) = mpsc::channel(pool_buffer);
        EmitterPool{ rx, map: HashMap::new(), client_buffer }.recv_loop();

        sx
    }

    fn recv_loop(mut self) {
        tokio::spawn(async move{
            loop{
                let msg = match self.rx.recv().await{
                    Some(msg) => msg,
                    //this happens when every emitter has been dropped
                    None => return,
                };
                self.handle_msg(msg).await;
                tokio::task::yield_now().await;
            }
        });
    }

    async fn handle_msg(&mut self, msg: PoolMessage<Res>) {
        match msg {
            PoolMessage::Connect(stream, id) => {
                let (sx, rx) = mpsc::channel(self.client_buffer);
                instance::Emitter::spawn_on_task(stream, rx);
                self.map.insert(id, sx);
            },
            PoolMessage::Msg(res, target) => self.send(res, target).await,
            PoolMessage::Disconnect(id) => { self.map.remove(&id); },
        }
    }

    async fn send(&mut self, res: Res, target: Target) {
        match target {
            #[cfg(feature = "broadcast")]
            Target::All => {
                for (_, sender) in self.map.iter() {
                    let res = res.clone();
                    sender.send(res).await.expect("instance::collectors shouldnt drop");
                }
            },
            Target::One(id) => 
                if let Some(sender) = self.map.get(&id) {
                    sender.send(res).await.expect("instance::collectors shouldnt drop");
                },
        }
    }
}

#[derive(Debug)]
pub(crate) enum PoolMessage<Msg>{
    Connect(OwnedWriteHalf, usize),
    Msg(Msg, Target),
    Disconnect(usize),
}