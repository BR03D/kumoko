mod instance;
use std::fmt::Debug;

pub mod server;
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

impl<T: Into<usize>> From<T> for Origin{
    fn from(id: T) -> Self {
        Self::Id(id.into())
    }
}