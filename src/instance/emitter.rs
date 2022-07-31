use std::io::{self, ErrorKind};

use tokio::{net::tcp::OwnedWriteHalf, sync::mpsc};

use crate::Message;

pub struct Emitter<Msg>{
    stream: OwnedWriteHalf,
    rx: mpsc::Receiver<Msg>,
}

impl<Msg: Message> Emitter<Msg> {
    pub fn spawn_on_task(
        stream: OwnedWriteHalf, 
        rx: mpsc::Receiver<Msg>,
    ) {
        Emitter{stream, rx}.emit_loop();
    }

    fn emit_loop(mut self) {
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
        let config = bincode::config::standard();
        let bin = bincode::encode_to_vec(msg, config).expect("how did this go wrong?");

        self.stream.writable().await?;
        self.stream.try_write(&bin)?;
    
        Ok(())
    }
}