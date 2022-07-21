use std::io;

use tokio::{net::{TcpListener, ToSocketAddrs}, sync::mpsc::{self, error::SendError}};
use crate::{Message, Origin, instance};

mod pool;
use pool::{PoolMessage, SenderPool};

///Initializes the accept loop, returning a Reciever and Sender.
/// 
/// They live on the main task, the Sender is Clone.
pub fn bind<I, Req: Message, Res: Message>(
    ip: I
) -> io::Result<(Receiver<Req>, Sender<Res>)> 
    where I: ToSocketAddrs + Send + 'static,
{
    let (sx, rx) = mpsc::channel(32);
    let pool = SenderPool::spawn_on_task();

    accept_loop(ip, sx, pool.clone())?;

    Ok((Receiver{rx}, Sender{pool}))
}

/// Lives on the main task
/// 
/// server::bind() will create one for you.
#[derive(Debug)]
pub struct Receiver<Req>{
    rx: mpsc::Receiver<(Req, Origin)>
}

impl<Req: Message> Receiver<Req> {
    /// Gets the next request if one is available, otherwise it waits until it is.
    pub async fn get_request(&mut self) -> (Req, Origin) {
        self.rx.recv().await.unwrap()
    }
}

/// Lives on the main task
/// 
/// server::bind() will create one for you.
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
    OnClient,
}

impl<T: Into<usize>> From<T> for Target{
    fn from(id: T) -> Self {
        Self::One(id.into())
    }
}

impl From<Origin> for Target{
    fn from(o: Origin) -> Self {
        match o {
            Origin::Id(i) => Self::One(i),
            Origin::OnClient => Self::OnClient,
        }
    }
}

fn accept_loop<I, Req: Message, Res: Message>(
    ip: I,
    sx:   mpsc::Sender<(Req, Origin)>, 
    pool: mpsc::Sender<PoolMessage<Res>>,
) -> io::Result<()> 
    where I: ToSocketAddrs + Send + 'static,
{
    let mut id = 0;
    
    tokio::spawn(async move{
        let listener = TcpListener::bind(ip).await.unwrap();
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