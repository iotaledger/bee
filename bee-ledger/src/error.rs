// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::transaction::OutputId, Error as MessageError, MessageId};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error {0}")]
    Io(std::io::Error),
    #[error("")]
    Message(MessageError),
    #[error("")]
    MissingMessage(MessageId),
    #[error("")]
    UnsupportedInputType,
    #[error("Message was not found")]
    MilestoneMessageNotFound,
    #[error("Message payload was not a milestone")]
    NoMilestonePayload,
    #[error("Tried to confirm {0} on top of {1}")]
    NonContiguousMilestone(u32, u32),
    #[error("The computed merkle proof on milestone {0} does not match the one provided by the coordinator {1}")]
    MerkleProofMismatch(String, String),
    #[error("Invalid messages count: referenced ({0}) != no transaction ({1}) + conflicting ({2}) + included ({3})")]
    InvalidMessagesCount(usize, usize, usize, usize),
    #[error("")]
    OutputNotFound(OutputId),
    #[error("")]
    Storage(Box<dyn std::error::Error + Send>),
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
