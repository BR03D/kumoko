use std::{io::{self, ErrorKind}, time::Duration};

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{Message, event::{Origin, Event, Illegal}};

pub struct Receiver<Msg: Message>{
    stream: OwnedReadHalf,
    sx: mpsc::Sender<(Event<Msg>, Origin)>,
    id: Origin,
    timeout: Duration,
}

impl<Msg: Message> Receiver<Msg>{
    pub fn spawn_on_task(
        stream: OwnedReadHalf, 
        sx: mpsc::Sender<(Event<Msg>, Origin)>, 
        id: Origin,
        timeout: Duration,
    ) {
        Receiver{stream, sx, id, timeout}.receive_loop();
    }

    fn receive_loop(self) {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;

                tokio::select! {
                    biased;
                    _ = self.sx.closed() => { return Ok::<(), ()>(()) }
                    _ = tokio::time::sleep(self.timeout) => { return Ok::<(), ()>(()) }

                    data = self.receive_data() => {
                        match data {
                            Ok(End::No) => (),
                            Ok(End::Yes) => {
                                self.send_event(Event::clean()).await?
                            },
                            Err(err) => match err.kind() {
                                ErrorKind::WouldBlock => (),
                                ErrorKind::ConnectionReset => {
                                    self.send_event(Event::dirty()).await?
                                },
                                _ => {
                                    self.send_event(Event::from_err(err)).await?
                                },
                            },
                        }
                    }
                };
            }
        });
    }

    async fn send_event(&self, event: Event<Msg>) -> Result<(), ()> {
        match self.sx.send((event, self.id)).await{
            Ok(_)  => Ok(()),
            // Dropped the Receiver!
            Err(_) => Err(()),
        }
    }

    async fn receive_data(&self) -> io::Result<End> {
        self.stream.readable().await?;
        const SIZE: usize = 256;

        let mut buf = [0; SIZE];
        let bytes_read = self.stream.try_read(&mut buf)?;

        match bytes_read {
            0 => return Ok(End::Yes),
            SIZE => self.receive_vec(&buf).await,
            n => self.decode_loop(&buf [..n]).await,
        }
    }

    async fn receive_vec(&self, buf: &[u8]) -> io::Result<End> {
        let mut vec = Vec::new();

        if let Err(e) = self.stream.try_read(&mut vec){
            // if exactly 256 bytes are sent, this will happen:
            if e.kind() == io::ErrorKind::WouldBlock {
                return self.decode_loop(&buf).await;
            }
            else { return Err(e) }
        };
        vec = [&buf [..], &vec].concat();
        self.decode_loop(&vec).await
    }

    async fn decode_loop(&self, data: &[u8]) -> io::Result<End> {
        let mut slice = &data[..];
        let config = bincode::config::standard();

        while slice.len() > 0{
            let res = bincode::decode_from_std_read::<Msg,_,_>(&mut slice, config);

            let i =  match res{
                Ok(msg) => self.send_event(msg.into()).await,
                Err(err) => self.send_event(Illegal{vec: Vec::from(slice), err: err.into()}.into()).await,
            };
            if let Err(()) = i{ return Ok(End::Yes) }
        };

        return Ok(End::No)
    }
}

enum End{
    Yes,
    No,
}