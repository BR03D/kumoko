use project_g::client;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = client::connect("188.34.181.80:1337").await?;

    send.send_request("Jack Jones".to_string()).await;
    let response: String = rec.get_response().await;
    println!("{}", response);

    Ok(())
}