use std::{io, time::Duration};

use tokio::{net::{TcpListener, ToSocketAddrs}, sync::{mpsc, watch}};
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
    pub async fn bind<I>(ip: I) -> io::Result<Server<Req, Res>>
        where I: ToSocketAddrs + Send + 'static,
    {
        Self::bind_with_config(ip, Config::default()).await
    }
    ///Initializes the accept loop, returning a Server. It can be 
    /// split into a Reciever and Sender for async operations.
    pub async fn bind_with_config<I>(ip: I, config: Config) -> io::Result<Server<Req, Res>>
        where I: ToSocketAddrs + Send + 'static,
    {
        let (sx, rx) = mpsc::channel(config.receiver_buffer);
        let pool = SenderPool::spawn_on_task(config.pool_buffer, config.client_buffer);
        let (_watch_sender, watch) = watch::channel(());
        let listener = TcpListener::bind(ip).await?;
    
        accept_loop(listener, sx, pool.clone(), watch, config.timeout)?;
        let receiver = Receiver{rx, pool: pool.clone(), _watch_sender};
        let sender = Sender{pool};
    
        Ok(Server{receiver, sender})
    }

    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        self.receiver.get_event().await
    }

    /// Convenience method for applications which only care about requests
    pub async fn get_request(&mut self) -> (Req, Origin) {
        self.receiver.get_request().await
    }

    /// Default method for streaming to Clients.
    pub async fn send_response(&self, res: Res, target: Target) {
        self.sender.send_response(res, target).await;
    }

    #[cfg(feature = "broadcast")]
    /// broadcast to every connected Client.
    pub async fn broadcast(&self, res: Res) {
        self.send_response(res, Target::All).await;
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
    pool: mpsc::Sender<PoolMessage<Res>>,
    _watch_sender: watch::Sender<()>,
}

impl<Req: Message, Res: Message> Receiver<Req, Res> {
    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        // the accept loop owns a sender and it only drops once this drops.
        let (e, o) = self.rx.recv().await.unwrap();
        if let (Event::Disconnect(_), Origin::Id(id)) = (&e, o) {
            // while this owns a sender, the pool cant drop
            self.pool.send(PoolMessage::Disconnect(id)).await.unwrap();
        }

        (e, o)
    }

    /// Convenience method for applications which only care about requests
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
    pub async fn send_response(&self, res: Res, target: Target) {
        self.pool.send(PoolMessage::Msg(res, target)).await.unwrap();
    }

    /// broadcast to every connected Client.
    #[cfg(feature = "broadcast")]
    pub async fn broadcast(&self, res: Res) {
        self.send_response(res, Target::All).await;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Target{
    #[cfg(feature = "broadcast")]
    All,
    One(usize),
}

/// Config for the Server
pub struct Config{
    /// If no new requests appear within this duration, we drop the client.
    timeout: Duration,
    /// The size of the channel buffer per Client Sender.
    client_buffer: usize,
    /// the size of the channel buffer for the receiver.
    receiver_buffer: usize,
    /// The size of the channel buffer for the SenderPool.
    pool_buffer: usize,
}

impl Default for Config{
    fn default() -> Config {
        Config { timeout: Duration::MAX, client_buffer: 3, receiver_buffer: 32, pool_buffer: 32 }
    }
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

fn accept_loop<Req: Message, Res: Message>(
    listener: TcpListener,
    sx:   mpsc::Sender<(Event<Req>, Origin)>, 
    pool: mpsc::Sender<PoolMessage<Res>>,
    mut watch: watch::Receiver<()>,
    timeout: Duration,
) -> io::Result<()> {
    let mut id = 0;
    
    tokio::spawn(async move{
        loop{
            tokio::select! {
                _ = watch.changed() => {
                    println!("Kumoko: Shutting down accept loop");
                    return Ok(())
                }
                res = listener.accept() => {
                    let (stream, _) = res?;
                    let (read, write) = stream.into_split();

                    instance::Receiver::spawn_on_task(read, sx.clone(), id.into(), timeout);

                    pool.send(PoolMessage::Connect(write, id)).await.unwrap();
                    sx.send((Event::Connect, id.into())).await.unwrap();
            
                    id += 1;    
                }
            };
            tokio::task::yield_now().await;
            }
        #[allow(unreachable_code)]
        Ok::<(), io::Error>(())
    });

    Ok(())
}