# Kumoko

A simple asynchronous server/client crate built 
on tokio for easy two-way streaming.

[Website](https://www.youtube.com/watch?v=dQw4w9WgXcQ) |
[API Docs](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

## Unstable Warning!
* Very early in development
* The documentation sucks
* .unwrap() is everywhere

## Motivation
Enable full duplex streaming of semi-complex data-structures between a rust server and clients. gRPC implementations are suboptimal for this:

* Unnecessary complexity
* Annoying Protocol buffers
* Restricted Data e.g. no enums

## Features
* Many Clients can communicate with the Server asynchronously
* Every Client has a full duplex connection
* Any data structure that implements `Message` can be transmitted:
```rust
trait Message: Send + Clone + Serialize + DeserializeOwned + 'static
```

## Examples

In your Cargo.toml: 
```toml
[dependencies]
tokio = { version = "1.20.0", features = ["macros", "rt-multi-thread"] }
kumoko = "0.3"
```

**Minimal Client:**
```rust
use kumoko::client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = client::connect("[::1]:50052").await?;

    send.send_request("Ferris".to_string()).await;
    let msg: String = rec.get_response().await.unwrap();
    println!("{}", msg);

    Ok(())
}
```

**Minimal Server:**
```rust
use kumoko::server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = server::bind("[::1]:50052")?;

    loop{
        let (req, target): (String, _) = rec.get_request().await;

        let msg = format!("Hello {}! Happy to see you here!", req);
        send.send_single(msg, target.into()).await?;
    }
}
```