use kumoko::{client::Client, server::Server, CloseEvent, Event};

const IP: &str = "[::1]:50052";

#[tokio::test]
async fn events() {
    let mut server = Server::<i32, i32>::bind(IP).await.unwrap();
    let mut client = Client::connect(IP).await.unwrap();

    client.send_request(15).await;

    let (event, origin) = server.get_event().await;
    match event{
        Event::Message(req) => {
            server.send_single(req + 4, origin.into()).await.unwrap();
        },
        Event::IllegalData(_) => unimplemented!(),
        Event::Close(c) => {
            assert_eq!(c, CloseEvent::Clean);
            return
        },
    }
    let res: i32 = client.get_response().await.unwrap();
    
    assert_eq!(res, 19);
}