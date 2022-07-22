use project_g::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = server::bind("[::1]:50052")?;

    loop{
        let (req, target): (String, _) = server.get_request().await;

        let msg = format!("Hello {}! Happy to see you here!", req);
        server.send_single(msg, target.into()).await?;
    }
}