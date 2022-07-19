use tokio::sync::mpsc::error::SendError;

use crate::responder::ResponderMessage;

#[derive(Debug)]
pub enum MyError{
    Io(std::io::Error),
    Serde(bincode::ErrorKind),
    Mpsc(SendError<ResponderMessage>),
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

impl From<SendError<ResponderMessage>> for MyError {
    fn from(err: SendError<ResponderMessage>) -> Self {
        Self::Mpsc(err)
    }
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyError: {:?}", self)
    }
}