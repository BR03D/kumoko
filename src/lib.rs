pub use bincode::{Decode, Encode};
pub use event::{Origin, Event, DisconnectEvent, Illegal};

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "broadcast")]
/// Any data structure implementing this can be transmitted. 
/// Usually, `#[derive(Decode, Encode)]` will be enough to implement it. 
/// If the broadcast feature is enabled, Clone is also required. 
/// 
/// Note: Debug is required for now. This will likely change in the future.
pub trait Message:              Send + Clone + fmt::Debug + Encode + Decode + 'static{}
#[cfg(feature = "broadcast")]
impl<T> Message for T where T:  Send + Clone + fmt::Debug + Encode + Decode + 'static{}

#[cfg(not(feature = "broadcast"))]
/// Any data structure implementing this can be transmitted. 
/// Usually, `#[derive(Decode, Encode)]` will be enough to implement it. 
/// If the broadcast feature is enabled, Clone is also required. 
/// 
/// Note: Debug is required for now. This will likely change in the future.
pub trait Message:              Send + fmt::Debug + Encode + Decode + 'static{}
#[cfg(not(feature = "broadcast"))]
impl<T> Message for T where T:  Send + fmt::Debug + Encode + Decode + 'static{}

mod event;
mod instance;
use std::fmt;