use std::error::Error;

mod responder;
mod server;
mod events;
mod database;
mod client_requester;
mod my_error;
use events::Request;
pub use my_error::MyError;

use crate::events::Response;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut server = server::Server::bind("0.0.0.0:1337").await;

    loop{
        let req = server.get_request().await;
        println!("{:?}", req);
        match req.msg {
            Request::MapRequest => {
                let map = database::get_map().await.unwrap();
                let msg = Response::SendMap(map);
                server.send_single(msg, req.target.into()).await;
            },
            Request::SaveMap(m) => {
                database::save_map(m).await.unwrap();
            },
        }
        tokio::task::yield_now().await;
    };
}