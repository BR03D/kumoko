use project_g::Server;

#[tokio::main]
async fn main() {
    let mut server = Server::bind("0.0.0.0:1337").await;

    loop{
        let req = server.get_request().await;
        let target = req.target;
        let msg: String = req.msg;

        let msg = format!("Hello {}! Happy to see you here!", msg);
        server.send_single(msg, target);

    }
}