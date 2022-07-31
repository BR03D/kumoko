use kumoko::{client::Client, server::Server, Encode, Decode};

const IP: &str = "[::1]:50052";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::<Login, Correct>::bind(IP).await.unwrap();
    tokio::spawn(async move{
        client().await;
    });

    loop{
        let (req, o) = server.get_request().await;
        if req.username == "Ferris" && req.password == "[rab$Rav3" {
            server.emit_response(Correct::Yes, o.into()).await;
        } else {
            server.emit_response(Correct::No,  o.into()).await;
        }
    }
}

async fn client() {
    let mut client = Client::<Login, Correct>::connect(IP).await.unwrap();

    // simulate some login attempts
    let req = Login{username: "Ferris".to_string(), password: "crabrave".to_string()};
    client.emit_request(req).await;
    let res = client.get_response().await.unwrap();
    println!("first: {:?}", res);

    let req = Login{username: "Ferris".to_string(), password: "[rab$Rav3".to_string()};
    client.emit_request(req).await;
    let res = client.get_response().await.unwrap();
    println!("second: {:?}", res);
}

#[derive(Debug, Clone, Encode, Decode)]
struct Login{
    username: String,
    password: String,
}

#[derive(Debug, Clone, Encode, Decode)]
enum Correct{
    Yes,
    No,
}