use project_g::{client, server};

const IP: &str = "[::1]:50052";

#[tokio::test]
async fn many() {
    let mut server = server::bind(IP).await.unwrap();
    create().await;

    loop{
        let (req, origin): (i32, _) = server.get_request().await;
        server.send_single(req + 1, origin.into()).await.unwrap();

        println!("sending {} to {}", req, origin);

        if req == 10 {return}
    }

}

async fn create() {
    for _ in 0..10{
        let mut client = client::connect(IP).await.unwrap();

        tokio::spawn(async move{
            let mut idx = 1;
            loop{
                client.send_request(idx).await;
                idx = client.get_response().await.unwrap();
                tokio::task::yield_now().await;
            }
        });
    };
}