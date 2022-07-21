use core::panic;
use std::io::{self, ErrorKind};

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{Origin, Message};

pub struct Receiver<Msg>{
    stream: OwnedReadHalf,
    sx: mpsc::Sender<(Msg, Origin)>,
    id: Origin
}

impl<Msg: Message> Receiver<Msg>{
    pub fn spawn_on_task(
        stream: OwnedReadHalf, 
        sx: mpsc::Sender<(Msg, Origin)>, 
        id: Origin
    ) {
        Receiver{stream, sx, id}.recieve_loop();
    }

    fn recieve_loop(self) {
        tokio::spawn(async move{
            loop{
                if let Err(e) = self.recieve().await{
                    match e.kind() {
                        ErrorKind::ConnectionReset => return,
                        ErrorKind::WouldBlock => continue,
                        _ => panic!("{}", e),
                    }
                };
                tokio::task::yield_now().await;
            }
        });
    }

    async fn recieve(&self) -> io::Result<()> {
        let multi = self.recieve_data().await?;

        match multi {
            MultiMessage::Single(req) => self.handle_request(req).await?,
            MultiMessage::Multi(vec) => {
                for req in vec{
                    self.handle_request(req).await?
                }
            },
        };

        Ok(())
    }

    async fn handle_request (&self, msg: Msg) -> io::Result<()> {

        self.sx.send((msg, self.id)).await.unwrap();

        Ok(())
    }

    async fn recieve_data(&self) -> io::Result<MultiMessage<Msg>> {
        self.stream.readable().await.unwrap();

        let mut buf = [0; 256];
        let bytes_read = self.stream.try_read(&mut buf)?;
        if bytes_read == 0 {return Err(ErrorKind::ConnectionReset.into())};
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
}