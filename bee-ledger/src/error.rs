// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::transaction::OutputId, Error as MessageError, MessageId};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Message(MessageError),
    MissingMessage(MessageId),
    UnsupportedInputType,
    MilestoneMessageNotFound,
    NoMilestonePayload,
    NonContiguousMilestone,
    MerkleProofMismatch,
    InvalidMessagesCount,
    OutputNotFound(OutputId),
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
