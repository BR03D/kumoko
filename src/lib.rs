pub use bincode::{Decode, Encode};

pub mod event;
pub use event::{Origin, Event, DisconnectEvent, Illegal};

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

pub trait Message:              Send + 'static + Clone + Debug + Encode + Decode{}
impl<T> Message for T where T:  Send + 'static + Clone + Debug + Encode + Decode{}

mod instance;
use std::fmt::Debug;