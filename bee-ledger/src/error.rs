// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{balance::Balance, model::Error as ModelError};

use bee_message::{milestone::MilestoneIndex, payload::transaction::Address, MessageId};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("")]
    Model(ModelError),
    #[error("Message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    #[error("")]
    UnsupportedInputType,
    #[error("")]
    UnsupportedOutputType,
    #[error("")]
    UnsupportedAddressType,
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
    #[error("Unexpected milestine diff index: {0:?}.")]
    UnexpectedDiffIndex(MilestoneIndex),
    #[error("Invalid ledger unspent state: {0}.")]
    InvalidLedgerUnspentState(u64),
    #[error("Invalid ledger balance state: {0}.")]
    InvalidLedgerBalanceState(u64),
    #[error("Invalid ledger dust state: {0:?} {1:?}.")]
    InvalidLedgerDustState(Address, Balance),
    #[error("")]
    Storage(Box<dyn std::error::Error + Send>),
}

impl From<ModelError> for Error {
    fn from(error: ModelError) -> Self {
        Error::Model(error)
    }
}
