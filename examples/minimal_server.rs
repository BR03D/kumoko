use project_g::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = server::bind("0.0.0.0:1337")?;

    loop{
        let (req, target): (String, _) = rec.get_request().await;

        let msg = format!("Hello {}! Happy to see you here!", req);
        send.send_single(msg, target.into()).await?;
    }
}