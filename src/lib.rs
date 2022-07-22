mod instance;
use std::fmt::Debug;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

pub trait Message:              Send + 'static + Clone + Debug + Serialize + DeserializeOwned{}
impl<T> Message for T where T:  Send + 'static + Clone + Debug + Serialize + DeserializeOwned{}

#[derive(Debug, Clone, Copy)]
pub enum Origin{
    Id(usize),
    OnClient,
}

#[derive(Debug, Clone)]
pub enum Event<Msg: Message>{
    Message(Msg),
    IllegalData(Vec<u8>),
    Close(CloseEvent)
}

#[derive(Debug, Clone)]
pub enum CloseEvent{
    Clean,
    Dirty,
}

impl<Msg: Message> Event<Msg> {
    fn clean() -> Self{
        Self::Close(CloseEvent::Clean)
    }

    fn dirty() -> Self{
        Self::Close(CloseEvent::Dirty)
    }
}

impl<Msg: Message> From<Msg> for Event<Msg> {
    fn from(msg: Msg) -> Self {
        Self::Message(msg)
    }
}


impl<T: Into<usize>> From<T> for Origin{
    fn from(id: T) -> Self {
        Self::Id(id.into())
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let st = match self {
            Origin::Id(id) => format!("{}",id),
            Origin::OnClient => "Server".to_string(),
        };
        write!(f, "{}", st)
    }
}