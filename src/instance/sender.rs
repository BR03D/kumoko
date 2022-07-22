use std::io::{self, ErrorKind};

use tokio::{net::tcp::OwnedWriteHalf, sync::mpsc};

use crate::Message;

pub struct Sender<Msg>{
    stream: OwnedWriteHalf,
    rx: mpsc::Receiver<Msg>,
}

impl<Msg: Message> Sender<Msg> {
    pub fn spawn_on_task(
        stream: OwnedWriteHalf, 
        rx: mpsc::Receiver<Msg>,
    ) {
        Sender{stream, rx}.send_loop();
    }

    fn send_loop(mut self) {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;
                let msg = match self.rx.recv().await{
                    Some(msg) => msg,

                    // this happens when the mpsc::sender is dropped - we simply end the loop
                    None => return,
                };

                if let Err(e) = self.respond(msg).await{
                    match e.kind() {
                        ErrorKind::WouldBlock => continue,

                        // this should never happen?
                        ErrorKind::BrokenPipe => panic!("In sender: {}", e),
                        _ => panic!("{}", e)
                    }
                };
            }
        });
    }

    async fn respond(&self, msg: Msg) -> io::Result<()> {
        self.stream.writable().await?;
        self.stream.try_write(&bincode::serialize(&msg).unwrap())?;
    
        Ok(())
    }
}