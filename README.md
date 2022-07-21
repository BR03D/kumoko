# Project_G

A simple, rust only asynchronous server/client crate built 
on tokio for easy full duplex streaming between rust programs.

[Website](https://www.youtube.com/watch?v=dQw4w9WgXcQ) |
[API Docs](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

## Unstable Warning!
* Very early in development
* The documentation sucks
* .unwrap() is everywhere

## Motivation
Enable full duplex streaming of semi-complex data-structures between a rust client and server. gRPC implementations are suboptimal for this:

* Unnecessary complexity
* Annoying Protocol buffers
* Restricted Data e.g. no enums

## Restrictions
All data structures must implement `Message`:
```rust
trait Message: Send + Clone + Serialize + DeserializeOwned + 'static
```

## Examples

**Minimal Client:**
```rust
use project_g::client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rec, send) = client::connect("[::1]:50052").await?;

    send.send_request("Rusty the Crab".to_string()).await;
    let msg: String = rec.get_response().await.unwrap();
    println!("{}", msg);

    Ok(())
}
```

**Minimal Server:**
```rust
use project_g::server;

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