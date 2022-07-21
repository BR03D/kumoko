use project_g::client;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = client::connect("188.34.181.80:1337").await?;

    send.send_request("Jack Jones".to_string()).await;
    let msg: String = rec.get_response().await.unwrap();
    println!("{}", msg);

    Ok(())
}