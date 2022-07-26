use kumoko::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::<String, String>::bind("[::1]:50052").await?;
    loop{
        let (req, origin) = server.get_request().await;

        let msg = format!("Hello {}! Happy to see you here!", req);
        server.send_response(msg, origin.into()).await;
    }
}