use kumoko::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect("[::1]:50052").await?;

    client.send_request("Ferris".to_string()).await;
    let msg: String = client.get_response().await.unwrap();
    println!("{}", msg);

    Ok(())
}