use std::io::{self, ErrorKind};

use tokio::{net::tcp::OwnedReadHalf, sync::mpsc};

use crate::{Origin, Message, Event, Illegal};

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
        Receiver{stream, sx, id}.receive_loop().ok();
    }

    fn receive_loop(self) -> Result<(), ()> {
        tokio::spawn(async move{
            loop{
                tokio::task::yield_now().await;
                
                match self.receive_data().await {
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
            #[allow(unreachable_code)]
            Ok::<(), ()>(())
        });
        Ok(())
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
            if let Err(()) = i{return Ok(End::Yes)}
        };

        return Ok(End::No)
    }
}

enum End{
    Yes,
    No,
}

/*
    Legacy code KEKW

    async fn handle_multi(&self, multi: MultiMessage<Msg>) {
        match multi {
            MultiMessage::Single(msg) => self.send_event(msg.into()).await,
            MultiMessage::Multi(v) =>
                for msg in v{
                    self.send_event(msg.into()).await
                },
            MultiMessage::Illegal(vec, err) => self.send_event(Illegal{vec, err}.into()).await,
            MultiMessage::CleanClose => self.send_event(Event::clean()).await,
        };
    }

    #[async_recursion::async_recursion]
    async fn receive_more_data(&self, data: &[u8]) -> io::Result<MultiMessage<Msg>> {
        let mut buf = [0; 1024];
        let bytes_read = self.stream.try_read(&mut buf)?;

        self.decode(&[data, &buf[..bytes_read]].concat()).await
    }

    if let Err(DecodeError::UnexpectedEnd) = res {
        return self.receive_more_data(data).await
    };


    

    async fn decode(&self, data: &[u8]) -> io::Result<MultiMessage<Msg>> {
        let mut slice = &data[..];
        
        let config = bincode::config::standard();
        let res = bincode::decode_from_std_read(&mut slice, config);

        let msg = match res{
            Ok(msg) => msg,
            Err(e) => return Ok(MultiMessage::Illegal(Vec::from(data), Arc::new(e))),
        };

        if slice.len() == 0 { 
            Ok(MultiMessage::Single(msg)) 
        }
        else {
            let first = vec![msg];
            self.decode_more(slice, first).await 
        }
    }

    #[async_recursion::async_recursion]
    async fn decode_more(&self, data: &[u8], mut before: Vec<Msg>) -> io::Result<MultiMessage<Msg>> {
        let mut slice = &data[..];

        let config = bincode::config::standard();
        let res = bincode::decode_from_std_read(&mut slice, config);

        let msg = match res{
            Ok(msg) => msg,
            Err(e) => return Ok(MultiMessage::Illegal(Vec::from(data), Arc::new(e))),
        };

        before.push(msg);
        
        if slice.len() == 0 {
            Ok(MultiMessage::Multi(before)) 
        }
        else {
            self.decode_more(slice, before).await 
        }
    }

    

#[derive(Debug)]
enum MultiMessage<Msg>{
    Single(Msg),
    Multi(Vec<Msg>),
    Illegal(Vec<u8>, Arc<DecodeError>),
    CleanClose,
}

*/