use kumoko::{client::Client, server::Server};

const IP: &str = "[::1]:50052";

#[tokio::main]
async fn main() {
    let mut server = Server::<i32, i32>::bind(IP).await.unwrap();
    let mut client = Client::connect(IP).await.unwrap();

    client.emit_request(15).await;

    let (req, origin) = server.get_request().await;
    server.emit_response(req + 4, origin.into()).await;

    let res: i32 = client.get_response().await.unwrap();

    println!("OUTPUT: {}", res);
}