// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::Error as MessageError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error {0}")]
    Io(std::io::Error),
    #[error("")]
    Message(MessageError),
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
