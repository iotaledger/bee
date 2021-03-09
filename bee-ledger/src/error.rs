// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Balance;

use bee_message::{address::Address, milestone::MilestoneIndex, Error as MessageError, MessageId};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
    #[error("")]
    Message(#[from] MessageError),
    #[error("Message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    #[error("")]
    UnsupportedInputType,
    #[error("")]
    UnsupportedOutputType,
    #[error("")]
    UnsupportedAddressType,
    #[error("")]
    UnsupportedTransactionEssenceType,
    #[error("")]
    UnsupportedPayloadType,
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
    #[error("Unexpected milestine diff index: {0:?}")]
    UnexpectedDiffIndex(MilestoneIndex),
    #[error("Invalid ledger unspent state: {0}")]
    InvalidLedgerUnspentState(u64),
    #[error("Invalid ledger balance state: {0}")]
    InvalidLedgerBalanceState(u64),
    #[error("Invalid ledger dust state: {0:?} {1:?}")]
    InvalidLedgerDustState(Address, Balance),
    #[error("Treasury amount mismatch: {0} != {1}")]
    TreasuryAmountMismatch(u64, u64),
    #[error("")]
    // TODO
    Option,
    #[error("")]
    Storage(Box<dyn std::error::Error + Send>),
}
