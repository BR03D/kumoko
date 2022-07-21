use std::error::Error;

mod database;
mod events;
use events::{Request, Response};

use project_g;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (mut rec,sender) = project_g::server::bind("0.0.0.0:1337")?;

    loop{
        let (req, target) = rec.get_request().await;
        println!("got {:?} from {:?}", req, target);
        match req {
            Request::MapRequest => {
                let map = database::get_map().await.unwrap();
                let msg = Response::SendMap(map);
                sender.send_single(msg, target.into());
            },
            Request::SaveMap(m) => {
                database::save_map(m).await.unwrap();
            },
        }
        tokio::task::yield_now().await;
    };
}