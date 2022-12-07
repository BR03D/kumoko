use kumoko::{client::Client, server::Server, event::Event::*};

const IP: &str = "[::1]:50052";

#[tokio::test]
async fn events() {
    let mut server = Server::<i32, i32>::bind(IP).await.unwrap();

    {
        let client = Client::<i64, i64>::connect(IP).await.unwrap();
        client.emit_request(i64::MAX).await;
    }
    {
        let client = Client::<i32, i32>::connect(IP).await.unwrap();
        client.emit_request(11111).await;
        client.emit_request(22222).await;
        client.emit_request(33333).await;
    }

    assert!(if let Connect        = server.get_event().await.0 {true} else {false});
    assert!(if let Connect        = server.get_event().await.0 {true} else {false});
    assert!(if let IllegalData(_) = server.get_event().await.0 {true} else {false});
    assert!(if let Message(11111) = server.get_event().await.0 {true} else {false});
    assert!(if let Disconnect(_)  = server.get_event().await.0 {true} else {false});
    assert!(if let Message(22222) = server.get_event().await.0 {true} else {false});
    assert!(if let Message(33333) = server.get_event().await.0 {true} else {false});
    assert!(if let Disconnect(_)  = server.get_event().await.0 {true} else {false});
}