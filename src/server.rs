use tokio::{net::TcpListener, sync::mpsc};

use crate::{responder::{Responder, ResponderMessage}, MyError, events::{Request, Response}, client_requester::ClientRequester};

pub struct Server{
    responder: mpsc::Sender<ResponderMessage>,
    requester: mpsc::Receiver<IRequest>,
}

impl Server {
    pub async fn bind(ip: &str) -> Server {
        let listener = TcpListener::bind(ip).await.unwrap();
        let (requester_sender, requester) = mpsc::channel(32);
        let responder = Responder::spawn_on_task();
        let clone = responder.clone();

        tokio::spawn(async move{
            accept_loop(listener, requester_sender, clone).await.unwrap()
        });

        Server{responder, requester}
    }

    pub async fn get_request(&mut self) -> IRequest {
        match self.requester.recv().await {
            Some(req) => req,
            None => panic!("WE CRASHED"),
        }
    }

    #[allow(unused)]
    pub async fn send_single(&self, msg: Response, target: Target) {
        self.send_response(IResponse{msg, target}).await;
    }

    #[allow(unused)]
    pub async fn broadcast(&self, msg: Response) {
        self.send_response(IResponse { msg, target: Target::All }).await;
    }

    pub async fn send_response(&self, res: IResponse) {
        self.responder.send(ResponderMessage::SendResponse(res)).await.unwrap();
    }
}
    
async fn accept_loop(
    listener: TcpListener, 
    requester_sender: mpsc::Sender<IRequest>,
    responder: mpsc::Sender<ResponderMessage>,
) -> Result<(), MyError> {
    let mut idx = 0;
    loop{
        let (stream, _) = listener.accept().await?;
        let (read, write) = stream.into_split();

        responder.send(
            ResponderMessage::AddConnection(write, idx)
        ).await.unwrap();

        ClientRequester::spawn_on_task(read, requester_sender.clone(), idx);
        idx += 1;
        tokio::task::yield_now().await;
    }
}

#[derive(Debug, Clone)]
pub struct IRequest{
    pub msg: Request,
    pub target: usize,
}

#[derive(Debug, Clone)]
pub struct IResponse{
    pub msg: Response,
    pub target: Target,
}

#[derive(Debug, Clone)]
pub enum Target{
    All,
    One(usize),
}

impl From<usize> for Target{
    fn from(idx: usize) -> Self {
        Self::One(idx)
    }
}