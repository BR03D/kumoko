use kumoko::{client::Client, server::Server, DisconnectEvent, Event};

const IP: &str = "[::1]:50052";

#[tokio::test]
#[ignore]
async fn events() {
    let mut server = Server::<i32, i32>::bind(IP).await.unwrap();
    let mut client = Client::connect(IP).await.unwrap();

    client.send_request(15).await;

    let (event, origin) = server.get_event().await;

    //doesnt do anything
    match event{
        Event::Message(req) => {
            server.send_response(req + 4, origin.into()).await;
        },
        Event::IllegalData(_) => unimplemented!(),
        Event::Disconnect(c) => {
            assert_eq!(c, DisconnectEvent::Clean);
            return
        },
        Event::Connect => (),
    }
    let res: i32 = client.get_response().await.unwrap();
    
    assert_eq!(res, 19);
}