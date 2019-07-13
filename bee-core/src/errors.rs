//! Errors

use std::io;

/// A specific Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A specific Error type.
#[derive(Debug)]
pub enum Error {
    /// A general application error.
    Node(&'static str),

    /// A channel send erro.
    EffectSend(crossbeam_channel::SendError<String>),

    /// A channel send erro.
    TriggerSend(tokio::sync::watch::error::SendError<bool>),

    /// An I/O error.
    Io(io::Error),
}

impl From<&'static str> for Error {
    fn from(msg: &'static str) -> Self {
        Error::Node(msg)
    }
}

impl From<crossbeam_channel::SendError<String>> for Error {
    fn from(e: crossbeam_channel::SendError<String>) -> Self {
        Error::EffectSend(e)
    }
}

impl From<tokio::sync::watch::error::SendError<bool>> for Error {
    fn from(e: tokio::sync::watch::error::SendError<bool>) -> Self {
        Error::TriggerSend(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
