use std::{sync::Arc, io};

use bincode::error::DecodeError;

use crate::Message;

/// Describes which client an `Event` originated from. `.into()`
/// can be used to transform into a `Target` to reply to.
#[derive(Debug, Clone, Copy)]
pub enum Origin{
    /// The Id of the Client.
    Id(usize),
    /// A Client can ignore this entirely.
    OnClient,
}

/// The Connection did something!
#[derive(Debug, Clone)]
pub enum Event<Msg: Message>{
    /// It connected!
    Connect,
    /// It sent a Message!
    Message(Msg),
    /// It sent Illegal data!
    IllegalData(Illegal),
    /// It disconnected!
    Disconnect(DisconnectEvent),
    /// An Error which didnt break the connection occured.
    RealError(Arc<io::Error>),
}

/// The connection was broken by:
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DisconnectEvent{
    /// a call to close it.
    Clean,
    /// an error.
    Dirty,
}

/// The sent Message couldnt be decoded. Includes the raw bytes and the `DecodeError`.
#[derive(Debug, Clone)]
pub struct Illegal{
    pub err: Arc<DecodeError>,
    pub vec: Vec<u8>,
}

impl<Msg: Message> Event<Msg> {
    pub(crate) fn clean() -> Self{
        Self::Disconnect(DisconnectEvent::Clean)
    }

    pub(crate) fn dirty() -> Self{
        Self::Disconnect(DisconnectEvent::Dirty)
    }

    pub(crate) fn from_err(err: io::Error) -> Self{
        Self::RealError(Arc::new(err))
    }
}

impl<Msg: Message> From<Msg> for Event<Msg> {
    fn from(msg: Msg) -> Self {
        Self::Message(msg)
    }
}

impl<Msg: Message> From<Illegal> for Event<Msg> {
    fn from(i: Illegal) -> Self {
        Self::IllegalData(i)
    }
}

impl From<(Vec<u8>, bincode::error::DecodeError)> for Illegal {
    fn from((vec, err): (Vec<u8>, bincode::error::DecodeError)) -> Self {
        Self{ vec, err: Arc::new(err) }
    }
}

impl<U: Into<usize>> From<U> for Origin{
    fn from(id: U) -> Self {
        Self::Id(id.into())
    }
}