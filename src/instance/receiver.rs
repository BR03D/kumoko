use core::panic;
use std::io::{self, ErrorKind};

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{Origin, Message, Event};

pub struct Receiver<Msg: Message>{
    stream: OwnedReadHalf,
    sx: mpsc::Sender<(Event<Msg>, Origin)>,
    id: Origin
}

impl<Msg: Message> Receiver<Msg>{
    pub fn spawn_on_task(
        stream: OwnedReadHalf, 
        sx: mpsc::Sender<(Event<Msg>, Origin)>, 
        id: Origin
    ) {
        Receiver{stream, sx, id}.recieve_loop();
    }

    fn recieve_loop(self) {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;
                match self.recieve_data().await{
                    Ok(multi) => self.handle_multi(multi).await,
                    Err(err) => self.handle_error(err).await,
                };
            }
        });
    }

    async fn handle_multi(&self, multi: MultiMessage<Msg>) {
        match multi {
            MultiMessage::Single(msg) => self.send_event(msg.into()).await,
            MultiMessage::Multi(v) =>
                for msg in v{
                    self.send_event(msg.into()).await
                },
            MultiMessage::CleanClose => self.send_event(Event::clean()).await,
        };
    }

    async fn handle_error(&self, err: io::Error) {
        match err.kind() {
            ErrorKind::ConnectionReset => {
                self.send_event(Event::dirty()).await;
            },
            ErrorKind::WouldBlock => (),
            _ => panic!("{}", err),
        }
    }

    async fn send_event(&self, event: Event<Msg>) {
        self.sx.send((event, self.id)).await.unwrap();        
    }

    async fn recieve_data(&self) -> io::Result<MultiMessage<Msg>> {
        self.stream.readable().await.unwrap();

        let mut buf = [0; 256];
        let bytes_read = self.stream.try_read(&mut buf)?;
        if bytes_read == 0 {
            return Ok(MultiMessage::CleanClose)
        };
        self.deserialize(&buf [..bytes_read]).await
    }

    #[async_recursion::async_recursion]
    async fn recieve_more_data(&self, data: &[u8]) -> io::Result<MultiMessage<Msg>> {
        let mut buf = [0; 1024];
        let bytes_read = self.stream.try_read(&mut buf)?;

        self.deserialize(&[data, &buf[..bytes_read]].concat()).await
    }

    async fn deserialize(&self, data: &[u8]) -> io::Result<MultiMessage<Msg>> {
        let mut slice = &data[..];
        let res = bincode::deserialize_from(&mut slice);

        if is_unexpected_eof(&res) {
            return self.recieve_more_data(data).await
        };

        let res = res.unwrap();

        if slice.len() == 0 { 
            Ok(MultiMessage::Single(res)) 
        }
        else { 
            let first = vec![res];
            self.deserialize_more(slice, first).await 
        }
    }

    #[async_recursion::async_recursion]
    async fn deserialize_more(&self, data: &[u8], mut before: Vec<Msg>) -> io::Result<MultiMessage<Msg>> {
        let mut slice = &data[..];
        let res = bincode::deserialize_from(&mut slice);

        if is_unexpected_eof(&res) {
            return self.recieve_more_data(data).await
        };

        let res = res.unwrap();

        before.push(res);
        
        if slice.len() == 0 { Ok(MultiMessage::Multi(before)) }
        else { self.deserialize_more(slice, before).await }
    }
}

fn is_unexpected_eof<Msg> (res: &Result<Msg, Box<bincode::ErrorKind>>) -> bool {
    if let Err(e) = res {
        if let bincode::ErrorKind::Io(err) = e.as_ref() {
            if let std::io::ErrorKind::UnexpectedEof = err.kind() {
                return true
            }
        }
    };
    false
}

#[derive(Debug)]
enum MultiMessage<Msg>{
    Single(Msg),
    Multi(Vec<Msg>),
    CleanClose,
}