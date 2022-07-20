mod responder;
mod server;
mod client_requester;
mod my_error;

pub use server::Server;
pub use serde::{Serialize, Deserialize};

pub trait TraitRequest: Send + 'static + std::fmt::Debug + serde::de::DeserializeOwned{}
impl<T> TraitRequest for T where T: Send + 'static + std::fmt::Debug + serde::de::DeserializeOwned{}

pub trait TraitResponse: Send + 'static + Clone + serde::Serialize + std::fmt::Debug {}
impl<T> TraitResponse for T where T: Send + 'static + Clone + serde::Serialize + std::fmt::Debug {}