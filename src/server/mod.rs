use std::io;

use tokio::{net::{TcpListener, ToSocketAddrs}, sync::mpsc::{self, error::SendError}};
use crate::{Message, Origin, instance, Event};

mod pool;
use pool::{PoolMessage, SenderPool};

pub struct Server<Req: Message, Res: Message>{
    receiver: Receiver<Req, Res>,
    sender: Sender<Res>,
}

impl<Req: Message, Res: Message> Server<Req, Res>{
    ///Initializes the accept loop, returning a Server. It can be 
    /// split into a Reciever and Sender for async operations.
    pub async fn bind<I>(
        ip: I
    ) -> io::Result<Server<Req, Res>>
        where I: ToSocketAddrs + Send + 'static,
    {
        let (sx, rx) = mpsc::channel(32);
        let pool = SenderPool::spawn_on_task();
        let listener = TcpListener::bind(ip).await?;
    
        accept_loop(listener, sx, pool.clone())?;
        let receiver = Receiver{rx, pool: pool.clone()};
        let sender = Sender{pool};
    
        Ok(Server{receiver, sender})
    }

    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        self.receiver.get_event().await
    }

    /// Convenience method for applications only care about requests
    pub async fn get_request(&mut self) -> (Req, Origin) {
        self.receiver.get_request().await
    }

    /// Default method for streaming to Clients.
    pub async fn send_response(&self, res: Res, target: Target) -> Result<(), SendError<PoolMessage<Res>>> {
        self.sender.send_response(res, target).await
    }

    /// broadcast to every connected Client.
    pub async fn broadcast(&self, res: Res) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response(res, Target::All).await
    }

    /// 
    pub fn into_split(self) -> (Receiver<Req, Res>, Sender<Res>) {
        (self.receiver, self.sender)
    }
}

/// Lives on the main task
/// 
/// Server.into_split will create one for you.
#[derive(Debug)]
pub struct Receiver<Req: Message, Res: Message>{
    rx: mpsc::Receiver<(Event<Req>, Origin)>,
    /// needs this to send disconnect messages to the pool
    pool: mpsc::Sender<PoolMessage<Res>>
}

impl<Req: Message, Res: Message> Receiver<Req, Res> {
    /// Gets the next event if one is available, otherwise it waits until it is.
    /// 
    /// We .unwrap() because the accept_loop owns a sender - it can never go out of 
    /// scope (that is maybe a problem)
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        let (e, o) = self.rx.recv().await.unwrap();
        if let (Event::Disconnect(_), Origin::Id(id)) = (e.clone(), o) {
            self.pool.send(PoolMessage::Disconnect(id)).await.unwrap();
        }

        (e, o)
    }

    /// Convenience method for applications only care about requests
    pub async fn get_request(&mut self) -> (Req, Origin) {
        loop{
            if let (Event::Message(msg), o) = self.get_event().await{
                return (msg, o)
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

    /// Default method for streaming to Clients.
    pub async fn send_response(&self, res: Res, target: Target) -> Result<(), SendError<PoolMessage<Res>>> {
        self.pool.send(PoolMessage::Msg((res, target))).await
    }

    /// broadcast to every connected Client.
    pub async fn broadcast(&self, res: Res) -> Result<(), SendError<PoolMessage<Res>>> {
        self.send_response(res, Target::All).await
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

impl<U: Into<usize>> From<U> for Target {
    fn from(id: U) -> Self {
        Self::One(id.into())
    }
}

/// Currently no way to end the accept loop - if you repeatedly bind and close 
/// servers this will leak memory for now
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

            pool.send(PoolMessage::Connect(write, id)).await.unwrap();
            sx.send((Event::Connect, id.into())).await.unwrap();
    
            id += 1;
            tokio::task::yield_now().await;
        }
        #[allow(unused)]
        Ok::<(), io::Error>(())
    });

    Ok(())
}