// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::Error as MessageError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    InvalidVariant,
    InvalidVersion(u8, u8),
    NoDownloadSourceAvailable,
    Message(MessageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error happened: {}.", e),
            Error::InvalidVariant => write!(f, "Invalid variant read."),
            Error::InvalidVersion(expected, actual) => write!(f, "Invalid version read: {}, {}.", expected, actual),
            Error::NoDownloadSourceAvailable => write!(f, ""),
            Error::Message(_) => write!(f, ""),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<MessageError> for Error {
    fn from(error: MessageError) -> Self {
        Error::Message(error)
    }
}
