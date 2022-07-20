use tokio::{net::TcpListener, sync::mpsc};

use crate::{responder::{Responder, ResponderMessage}, my_error::MyError, client_requester::ClientRequester, TraitRequest, TraitResponse};

pub struct Server<Req, Res>{
    responder: mpsc::Sender<ResponderMessage<Res>>,
    requester: mpsc::Receiver<IRequest<Req>>,
}

impl<Req: TraitRequest, Res: TraitResponse> Server<Req, Res> {
    pub async fn bind(ip: &str) -> Server<Req, Res> {
        let listener = TcpListener::bind(ip).await.unwrap();
        let (requester_sender, requester) = mpsc::channel(32);
        let responder = Responder::spawn_on_task();
        let clone = responder.clone();

        tokio::spawn(async move{
            Server::accept_loop(listener, requester_sender, clone).await.unwrap()
        });

        Server{responder, requester}
    }

    pub async fn get_request(&mut self) -> IRequest<Req> {
        match self.requester.recv().await {
            Some(req) => req,
            None => panic!("WE CRASHED"),
        }
    }

    #[allow(unused)]
    pub fn send_single(&self, msg: Res, target: usize) {
        self.send_response(IResponse{msg, target: target.into()});
    }

    #[allow(unused)]
    pub fn broadcast(&self, msg: Res) {
        self.send_response(IResponse { msg, target: Target::All });
    }

    pub fn send_response(&self, res: IResponse<Res>) {
        self.responder.try_send(ResponderMessage::SendResponse(res)).unwrap();
    }
    
    async fn accept_loop(
        listener: TcpListener, 
        requester_sender: mpsc::Sender<IRequest<Req>>,
        responder: mpsc::Sender<ResponderMessage<Res>>,
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
}

#[derive(Debug, Clone)]
pub struct IRequest<Req>{
    pub msg: Req,
    pub target: usize,
}

#[derive(Debug, Clone)]
pub struct IResponse<Res>{
    pub msg: Res,
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