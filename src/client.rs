use std::io::ErrorKind;

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc::Sender};

use crate::{MyError, events::{Request, Response}, database, responder::ResponderMessage};

#[derive(Debug)]
pub struct Client {
    stream: OwnedReadHalf,
    responder: Sender<ResponderMessage>,
    index: usize,
}

impl Client {

    pub fn recieve_loop(stream: OwnedReadHalf, responder: Sender<ResponderMessage>, index: usize) {
        println!("Creating new client! We are number {}", index);
        let client = Client {stream, responder, index};

        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;
                let res = client.recieve().await;

                if let Err(MyError::Io(e)) = &res {
                    if e.kind() == ErrorKind::ConnectionReset {break;}
                };

                if let Err(MyError::Io(e)) = &res {
                    if e.kind() == ErrorKind::WouldBlock {continue;}
                }

                res.unwrap();
            }
            
        });
    }

    async fn recieve(&self) -> Result<(), MyError> {

        let multi = self.recieve_data().await?;

        match multi {
            MultiRequest::Single(req) => self.handle_request(req).await?,
            MultiRequest::Multi(vec) => {
                for req in vec{
                    self.handle_request(req).await?
                }
            },
        };

        Ok(())
    }
    async fn handle_request (&self, req: Request) -> Result<(), MyError> {
        println!("{:?}", req);
        match req {
            Request::MapRequest => {
                let map = database::get_map().await.unwrap();
                let resp = Response::SendMap(map);
                self.responder.send(ResponderMessage::SendResponse(resp, self.index)).await.unwrap();
            },
            Request::SaveMap(m) => {
                database::save_map(m).await.unwrap();
            },
        };
        Ok(())
    }

    async fn recieve_data(&self) -> Result<MultiRequest, MyError> {
        self.stream.readable().await.unwrap();

        let mut buf = [0; 256];
        let bytes_read = self.stream.try_read(&mut buf)?;
        self.deserialize(&buf [..bytes_read]).await
    }

    #[async_recursion::async_recursion]
    async fn recieve_more_data(&self, data: &[u8]) -> Result<MultiRequest, MyError> {
        let mut buf = [0; 1024];
        let bytes_read = self.stream.try_read(&mut buf)?;

        self.deserialize(&[data, &buf[..bytes_read]].concat()).await
    }

    async fn deserialize(&self, data: &[u8]) -> Result<MultiRequest, MyError> {
        let mut slice = &data[..];
        let res = bincode::deserialize_from(&mut slice);

        if is_unexpected_eof(&res) {
            return self.recieve_more_data(data).await
        };

        if slice.len() == 0 { 
            Ok(MultiRequest::Single(res?)) 
        }
        else { 
            let first = vec![res?];
            self.deserialize_more(slice, first).await 
        }
    }

    #[async_recursion::async_recursion]
    async fn deserialize_more(&self, data: &[u8], mut before: Vec<Request>) -> Result<MultiRequest, MyError> {
        let mut slice = &data[..];
        let res = bincode::deserialize_from(&mut slice);

        if is_unexpected_eof(&res) {
            return self.recieve_more_data(data).await
        };

        before.push(res?);
        
        if slice.len() == 0 { Ok(MultiRequest::Multi(before)) }
        else { self.deserialize_more(slice, before).await }
    }
}

fn is_unexpected_eof (res: &Result<Request, Box<bincode::ErrorKind>>) -> bool {
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
enum MultiRequest{
    Single(Request),
    Multi(Vec<Request>),
}