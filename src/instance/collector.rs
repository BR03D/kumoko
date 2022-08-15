use std::{io::{self, ErrorKind}, time::Duration};

use bincode::{config::Configuration, error::DecodeError};
use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};
use crate::{Message, event::{Origin, Event, Illegal}};

use super::ring_buffer::RingBuffer;

pub struct Collector<Msg: Message>{
    stream: OwnedReadHalf,
    sx: mpsc::Sender<(Event<Msg>, Origin)>,
    id: Origin,
    timeout: Duration,
    buffer: RingBuffer,
    config: Configuration,
}

impl<Msg: Message> Collector<Msg>{
    pub fn spawn_on_task(
        stream: OwnedReadHalf, 
        sx: mpsc::Sender<(Event<Msg>, Origin)>, 
        id: Origin,
        timeout: Duration,
    ) {
        let config = bincode::config::standard();
        Collector{stream, sx, id, timeout, buffer:RingBuffer::new(), config}.collect_loop();
    }

    fn collect_loop(mut self) {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;
                let sx_clone = self.sx.clone();
                
                tokio::select! {
                    biased;
                    _ = sx_clone.closed() => { return }
                    _ = tokio::time::sleep(self.timeout) => { return }

                    data = self.collect_data() => {
                        match data {
                            Ok(Status::Finish) => return self.send_event(Event::clean()).await,
                            Ok(Status::Continue) => (),
                            Err(err) => match err.kind() {
                                ErrorKind::WouldBlock => (),
                                ErrorKind::ConnectionReset => return self.send_event(Event::dirty()).await,
                                _ => self.send_event(Event::from_err(err)).await,
                            },
                        }
                    }
                };
            }
        });
    }

    async fn send_event(&mut self, event: Event<Msg>) {
        // we check for server dropping above :)
        self.sx.send((event, self.id)).await.ok();
    }

    async fn collect_data(&mut self) -> io::Result<Status> {
        self.stream.readable().await?;

        let bytes_read = self.stream.try_read_buf(&mut self.buffer)?;
        
        match bytes_read {
            0 => Ok(Status::Finish),
            _ => {
                self.decode_loop().await;
                Ok(Status::Continue)
            },
        }
    }

    async fn decode_loop(&mut self) {
        loop {
            match bincode::decode_from_reader::<Msg,_,_>(&mut self.buffer, self.config){
                Ok(msg) => {
                    self.buffer.fwd();
                    self.send_event(msg.into()).await;
                },
                Err(err) => {
                    match err{
                        DecodeError::UnexpectedEnd => {
                            self.buffer.back();
                            return
                        }
                        _ => {
                            self.buffer.clear();
                            self.send_event(Illegal{vec: Vec::new(), err: err.into()}.into()).await;
                        }
                    }
                },
            };
        }
    }
}

enum Status {
    Finish,
    Continue,
}