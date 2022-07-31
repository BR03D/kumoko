use std::{io::{self, ErrorKind}, time::Duration};

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{Message, event::{Origin, Event, Illegal}};

pub struct Collector<Msg: Message>{
    stream: OwnedReadHalf,
    sx: mpsc::Sender<(Event<Msg>, Origin)>,
    id: Origin,
    timeout: Duration,
}

impl<Msg: Message> Collector<Msg>{
    pub fn spawn_on_task(
        stream: OwnedReadHalf, 
        sx: mpsc::Sender<(Event<Msg>, Origin)>, 
        id: Origin,
        timeout: Duration,
    ) {
        Collector{stream, sx, id, timeout}.collect_loop();
    }

    fn collect_loop(self) {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;

                tokio::select! {
                    biased;
                    _ = self.sx.closed() => { return Ok::<(), ()>(()) }
                    _ = tokio::time::sleep(self.timeout) => { return Ok(())}

                    data = self.collect_data() => {
                        match data {
                            Ok(Err(_)) => return self.send_event(Event::clean()).await,
                            Ok(Ok(_)) => (),
                            Err(err) => match err.kind() {
                                ErrorKind::WouldBlock => (),
                                ErrorKind::ConnectionReset => return self.send_event(Event::dirty()).await,
                                _ => self.send_event(Event::from_err(err)).await?,
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
            // Dropped the Collector!
            Err(_) => Err(()),
        }
    }

    async fn collect_data(&self) -> io::Result<Result<(), ()>> {
        self.stream.readable().await?;
        const SIZE: usize = 256;

        let mut buf = [0; SIZE];
        let bytes_read = self.stream.try_read(&mut buf)?;

        match bytes_read {
            0 => return Ok(Err(())),
            SIZE => self.collect_vec(&buf).await,
            n => Ok(self.decode_loop(&buf [..n]).await),
        }
    }

    async fn collect_vec(&self, buf: &[u8]) -> io::Result<Result<(), ()>> {
        let mut vec = Vec::new();

        if let Err(e) = self.stream.try_read(&mut vec){
            // if exactly 256 bytes are sent, this will happen:
            if e.kind() == io::ErrorKind::WouldBlock {
                return Ok(self.decode_loop(&buf).await)
            }
            else { return Err(e) }
        };
        vec = [&buf [..], &vec].concat();
        Ok(self.decode_loop(&vec).await)
    }

    async fn decode_loop(&self, data: &[u8]) -> Result<(), ()> {
        let mut slice = &data[..];
        let config = bincode::config::standard();

        while slice.len() > 0{
            let res = bincode::decode_from_std_read::<Msg,_,_>(&mut slice, config);

            match res{
                Ok(msg) => self.send_event(msg.into()).await?,
                Err(err) => {
                    self.send_event(Illegal{vec: Vec::from(slice), err: err.into()}.into()).await?;
                    return Ok(())
                },
            };
        };

        return Ok(())
    }
}