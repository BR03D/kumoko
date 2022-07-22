use std::io;

use tokio::{net::{TcpListener, ToSocketAddrs}, sync::mpsc::{self, error::SendError}};
use crate::{Message, Origin, instance, Event};

mod pool;
use pool::{PoolMessage, SenderPool};

///Initializes the accept loop, returning a Server. It can be 
/// split into a Reciever and Sender for async operations.
pub async fn bind<I, Req: Message, Res: Message>(
    ip: I
) -> io::Result<Server<Req, Res>>
    where I: ToSocketAddrs + Send + 'static,
{
    let (sx, rx) = mpsc::channel(32);
    let pool = SenderPool::spawn_on_task();
    let listener = TcpListener::bind(ip).await?;

    accept_loop(listener, sx, pool.clone())?;
    let receiver = Receiver{rx};
    let sender = Sender{pool};

    Ok(Server{receiver, sender})
}

pub struct Server<Req: Message, Res>{
    receiver: Receiver<Req>,
    sender: Sender<Res>,
}

impl<Req: Message, Res: Message> Server<Req, Res>{
    /// Gets the next request if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        self.receiver.get_event().await
    }

    pub async fn get_request(&mut self) -> (Req, Origin) {
        self.receiver.get_request().await
    }

    #[allow(unused)]
    pub async fn send_single(&self, res: Res, target: Target) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response((res, target)).await
    }

    #[allow(unused)]
    pub async fn broadcast(&self, res: Res) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response((res, Target::All)).await
    }

    pub async fn send_response(&self, msg: (Res, Target)) -> Result<(), SendError<PoolMessage<Res>>> {
        self.sender.send_response(msg).await
    }
    /// 
    pub fn into_split(self) -> (Receiver<Req>, Sender<Res>) {
        (self.receiver, self.sender)
    }
}

/// Lives on the main task
/// 
/// Server.into_split will create one for you.
#[derive(Debug)]
pub struct Receiver<Req: Message>{
    rx: mpsc::Receiver<(Event<Req>, Origin)>
}

impl<Req: Message> Receiver<Req> {
    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        self.rx.recv().await.unwrap()
    }

    pub async fn get_request(&mut self) -> (Req, Origin) {
        loop{
            match self.get_event().await {
                (Event::Message(msg), o) => return (msg, o),
                _ => continue,
            }
        }
    }
}

/// Lives on the main task
/// 
/// Server.into_split will create one for you. Implements Clone for 
/// your own async operations.
#[derive(Debug, Clone)]
pub struct Sender<Res>{
    pool: mpsc::Sender<PoolMessage<Res>>
}

impl<Res: Message> Sender<Res>{
    #[allow(unused)]
    pub async fn send_single(&self, res: Res, target: Target) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response((res, target)).await
    }

    #[allow(unused)]
    pub async fn broadcast(&self, res: Res) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response((res, Target::All)).await
    }

    pub async fn send_response(&self, msg: (Res, Target)) -> Result<(), SendError<PoolMessage<Res>>> {
        self.pool.send(PoolMessage::Msg(msg)).await
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Target{
    All,
    One(usize),
}

impl From<Origin> for Target{
    fn from(o: Origin) -> Self {
        match o {
            Origin::Id(i) => Self::One(i),
            Origin::OnClient => unreachable!(),
        }
    }
}

fn accept_loop<Req: Message, Res: Message>(
    listener: TcpListener,
    sx:   mpsc::Sender<(Event<Req>, Origin)>, 
    pool: mpsc::Sender<PoolMessage<Res>>,
) -> io::Result<()> {
    let mut id = 0;
    
    tokio::spawn(async move{
        loop{
            let (stream, _) = listener.accept().await?;
            let (read, write) = stream.into_split();

            instance::Receiver::spawn_on_task(read, sx.clone(), id.into());

            pool.send(PoolMessage::Join(write, id)).await.unwrap();
    
            id += 1;
            tokio::task::yield_now().await;
        }
        #[allow(unused)]
        Ok::<(), io::Error>(())
    });

    Ok(())
}