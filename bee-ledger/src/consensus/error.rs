// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{Balance, Error as TypesError};

use bee_message::{address::Address, milestone::MilestoneIndex, Error as MessageError, MessageId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("")]
    Message(#[from] MessageError),
    #[error("")]
    Types(#[from] TypesError),
    #[error("Message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    #[error("")]
    UnsupportedInputKind(u8),
    #[error("")]
    UnsupportedOutputKind(u8),
    #[error("")]
    UnsupportedAddressKind(u8),
    #[error("")]
    UnsupportedTransactionEssenceKind(u8),
    #[error("")]
    UnsupportedPayloadKind(u32),
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
    #[error("Consumed amount overflow: tried to add {0} and {1}.")]
    ConsumedAmountOverflow(u64, u64),
    #[error("Created amount overflow: tried to add {0} and {1}.")]
    CreatedAmountOverflow(u64, u64),
    #[error("")]
    Storage(Box<dyn std::error::Error + Send>),
}
