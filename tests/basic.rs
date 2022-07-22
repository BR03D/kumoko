use project_g::{client, server};

const IP: &str = "[::1]:50052";

#[tokio::test]
async fn basic() {
    let mut server = server::bind(IP).await.unwrap();
    let mut client = client::connect(IP).await.unwrap();

    client.send_request(15).await;

    let (req, origin): (i32, _) = server.get_request().await;
    server.send_single(req + 4, origin.into()).await.unwrap();

    let res: i32 = client.get_response().await.unwrap();
    
    assert_eq!(res, 19);
}