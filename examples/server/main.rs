use std::error::Error;

mod database;
mod events;
use events::{Request, Response};

use project_g::{server, Origin};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (mut rec,send) = server::bind("0.0.0.0:1337")?.into_split();

    loop{
        let (req, origin) = rec.get_request().await;
        println!("got {:?} from {:?}", req, origin);
        match req {
            Request::MapRequest => {
                handle_map_req(send.clone(), origin);
            },
            Request::SaveMap(m) => {
                tokio::spawn(async move{
                    database::save_map(m).await.unwrap();
                });
            },
        }
        tokio::task::yield_now().await;
    };
}

fn handle_map_req(send: server::Sender<Response>, from: Origin) {
    tokio::spawn(async move{
        let map = database::get_map().await.unwrap();
        let msg = Response::SendMap(map);
        send.send_single(msg, from.into()).await.unwrap();
    });
}