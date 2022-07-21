use project_g::client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = client::connect("[::1]:50052").await?;

    send.send_request("Rusty the Crab".to_string()).await;
    let msg: String = rec.get_response().await.unwrap();
    println!("{}", msg);

    Ok(())
}