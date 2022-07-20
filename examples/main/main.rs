use std::error::Error;

mod database;
mod events;
use events::{Request, Response};

use project_g;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut server = project_g::Server::bind("0.0.0.0:1337").await;

    loop{
        let req = server.get_request().await;
        println!("{:?}", req);
        match req.msg {
            Request::MapRequest => {
                let map = database::get_map().await.unwrap();
                let msg = Response::SendMap(map);
                server.send_single(msg, req.target);
            },
            Request::SaveMap(m) => {
                database::save_map(m).await.unwrap();
            },
        }
        tokio::task::yield_now().await;
    };
}