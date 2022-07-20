mod responder;
mod server;
mod client_requester;
mod my_error;

use std::fmt::Debug;
use serde::{Serialize as Se, de::DeserializeOwned as De};

#[cfg(feature = "server")]
pub use server::Server;

pub use serde::{Serialize, Deserialize};

pub trait Message:              Send + 'static + Clone + Debug + Se + De{}
impl<T> Message for T where T:  Send + 'static + Clone + Debug + Se + De{}