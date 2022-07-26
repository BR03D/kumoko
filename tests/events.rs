//#![feature(assert_matches)]
//use std::assert_matches::assert_matches;

//  , event::{DisconnectEvent, Event}

use kumoko::{client::Client, server::Server};

const IP: &str = "[::1]:50052";

#[tokio::test]
#[ignore]
async fn events() {
    let mut server = Server::<i32, i32>::bind(IP).await.unwrap();
    println!("\nGO!");

    {
        let client = Client::<i64, i64>::connect(IP).await.unwrap();
        client.send_request(i64::MAX).await;
    }
    {
        let client = Client::<i32, i32>::connect(IP).await.unwrap();
        client.send_request(1232).await;
        client.send_request(22).await;
        client.send_request(123456).await;
    }

    for _ in 0..8 {
        println!("{:?}", server.get_event().await.0);
        //assert_eq!(server.get_event().await.0, Event::Connect);
    }
}

/*
        match event{
            Event::Message(_) => (),
            Event::IllegalData(_) => unimplemented!(),
            Event::RealError(_) => unimplemented!(),
            Event::Disconnect(c) => {
                assert_eq!(c, DisconnectEvent::Clean);
                return
            },
            Event::Connect => (),
        }
*/