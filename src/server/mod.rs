use std::{io, time::Duration};

use tokio::{net::{TcpListener, ToSocketAddrs}, sync::mpsc};
use crate::{Message, instance, event::{Origin, Event}};

mod pool;
use pool::{PoolMessage, SenderPool};

#[derive(Debug)]
/// A Server with an asynchronous full-duplex connection with every 
/// Client. Can be into_split into a Reciever and Sender for async operations.
pub struct Server<Req: Message, Res: Message>{
    receiver: Receiver<Req, Res>,
    sender: Sender<Res>,
}

impl<Req: Message, Res: Message> Server<Req, Res>{
    /// Initializes the accept loop, returning a Server with the default Config.
    pub async fn bind<I>(ip: I) -> io::Result<Server<Req, Res>>
        where I: ToSocketAddrs + Send + 'static,
    {
        Self::bind_with_config(ip, Config::default()).await
    }

    /// Initializes the accept loop, returning a Server. The Config can be customized.
    pub async fn bind_with_config<I>(ip: I, config: Config) -> io::Result<Server<Req, Res>>
        where I: ToSocketAddrs + Send + 'static,
    {
        let (sx, rx) = mpsc::channel(config.receiver_buffer);
        let pool = SenderPool::spawn_on_task(config.pool_buffer, config.client_buffer);
        let listener = TcpListener::bind(ip).await?;
    
        accept_loop(listener, sx, pool.clone(), config.timeout)?;
        let receiver = Receiver{rx, pool: pool.clone()};
        let sender = Sender{pool};
    
        Ok(Server{receiver, sender})
    }

    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        self.receiver.get_event().await
    }

    /// Convenience method for applications which only care about requests.
    pub async fn get_request(&mut self) -> (Req, Origin) {
        self.receiver.get_request().await
    }

    /// Default method for streaming to Clients.
    pub async fn send_response(&self, res: Res, target: Target) {
        self.sender.send_response(res, target).await;
    }

    #[cfg(feature = "broadcast")]
    /// Broadcast to every connected Client.
    pub async fn broadcast(&self, res: Res) {
        self.send_response(res, Target::All).await;
    }

    /// Splits the Server into a Receiver and a Sender. The Sender can be 
    /// cloned for async operations.
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
}

impl<Req: Message, Res: Message> Receiver<Req, Res> {
    /// Gets the next event if one is available, otherwise it waits until it is.
    pub async fn get_event(&mut self) -> (Event<Req>, Origin) {
        // the accept loop owns a sender and it only drops once this drops.
        let (e, o) = self.rx.recv().await.expect("while this owns a sender, the pool wont drop");
        if let (Event::Disconnect(_), Origin::Id(id)) = (&e, o) {
            // while this owns a sender, the pool cant drop
            self.pool.send(PoolMessage::Disconnect(id)).await.expect("while this owns a sender, the pool wont drop");
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
        // while this owns a sender, the pool cant drop
        self.pool.send(PoolMessage::Msg(res, target)).await.expect("while this owns a sender, the pool wont drop");
    }

    /// Broadcast to every connected Client.
    #[cfg(feature = "broadcast")]
    pub async fn broadcast(&self, res: Res) {
        self.send_response(res, Target::All).await;
    }
}

#[derive(Debug, Clone, Copy)]
/// The Target of a Response.
pub enum Target{
    #[cfg(feature = "broadcast")]
    /// Respond to every connected Client. 
    /// 
    /// Equivalent to using .broadcast()
    All,
    /// Respond to a specific Client. Origin.into() can be used to create one of these.
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
    timeout: Duration,
) -> io::Result<()> {
    let mut id = 0;
    
    tokio::spawn(async move{
        loop{
            let (stream, _) = listener.accept().await?;
            let (read, write) = stream.into_split();

            instance::Receiver::spawn_on_task(read, sx.clone(), id.into(), timeout);

            pool.send(PoolMessage::Connect(write, id)).await.expect("while this owns a sender, the pool wont drop");
            if let Err(_) = sx.send((Event::Connect, id.into())).await{
                // happens 
                return Ok::<(), io::Error>(())
            };
    
            id += 1;
            tokio::task::yield_now().await;
        }
    });

    Ok(())
}