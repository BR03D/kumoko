use std::error::Error;
use events::Response;
use responder::{Responder, ResponderMessage};
use tokio::{net::TcpListener, sync::oneshot};

mod responder;

mod events;

mod database;

mod client;
use client::Client;

mod my_error;
pub use my_error::MyError;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {    
    let listener = TcpListener::bind("0.0.0.0:1337").await?;
    let responder = Responder::new();

    loop{
        let (stream, _) = listener.accept().await?;
        let (read, write) = stream.into_split();
        let (sx, rx) = oneshot::channel();

        responder.send(ResponderMessage::AddConnection(write, sx)).await.unwrap();

        let index = rx.await.unwrap();
        
        Client::recieve_loop(read, responder.clone(), index);
    };
}