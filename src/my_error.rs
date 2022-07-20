use tokio::sync::mpsc::error::SendError;

use crate::responder::ResponderMessage;

#[derive(Debug)]
pub enum MyError{
    Io(std::io::Error),
    Serde(bincode::ErrorKind),
    SendError,
}

impl From<Box<bincode::ErrorKind>> for MyError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        Self::Serde(*err)
    }
}

impl From<bincode::ErrorKind> for MyError {
    fn from(err: bincode::ErrorKind) -> Self {
        Self::Serde(err)
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl<R> From<SendError<ResponderMessage<R>>> for MyError {
    fn from(_err: SendError<ResponderMessage<R>>) -> Self {
        Self::SendError
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyError: {:?}", self)
    }
}