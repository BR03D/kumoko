/// Definitions for Connection Events
pub mod event;
pub use bincode::{Decode, Encode};

#[cfg(feature = "server")]
/// Module for Server functionality. Enable the server feature to use it.
pub mod server;

#[cfg(feature = "client")]
/// Module for Client functionality. Enable the client feature to use it.
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

mod instance;
use std::fmt;