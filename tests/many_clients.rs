use kumoko::{client::Client, server::Server};

const IP: &str = "[::1]:50052";

#[tokio::test]
async fn many() {
    let mut server = Server::bind(IP).await.unwrap();
    create().await;

    loop{
        let (req, origin): (i32, _) = server.get_request().await;
        server.emit_response(req + 1, origin.into()).await;

        println!("sending {} to {:?}", req, origin);

        if req == 10 {return}
    }

}

async fn create() {
    for _ in 0..10{
        let mut client = Client::connect(IP).await.unwrap();

        tokio::spawn(async move{
            let mut idx = 1;
            loop{
                client.emit_request(idx).await;
                idx = client.get_response().await.unwrap();
                tokio::task::yield_now().await;
            }
        });
    };
}