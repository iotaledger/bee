// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the errors that can occur during ledger operations.

use bee_message::{address::Address, milestone::MilestoneIndex, Error as MessageError, MessageId};

use crate::{
    types::{Balance, Error as TypesError, Unspent},
    workers::{pruning::error::PruningError, snapshot::error::Error as SnapshotError},
};

/// Errors occurring during ledger workers operations.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),
    #[error("Pruning error: {0}")]
    Pruning(#[from] PruningError),
    #[error("Types error: {0}")]
    Types(#[from] TypesError),
    #[error("Message error: {0}")]
    Message(#[from] MessageError),
    #[error("Message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    #[error("Unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    #[error("Unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    #[error("Unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    #[error("Milestone message not found: {0}")]
    MilestoneMessageNotFound(MessageId),
    #[error("Message payload is not a milestone")]
    NoMilestonePayload,
    #[error("Non contiguous milestones: tried to confirm {0} on top of {1}")]
    NonContiguousMilestones(u32, u32),
    #[error("Merkle proof mismatch on milestone {0}: computed {1} != provided {2}")]
    MerkleProofMismatch(MilestoneIndex, String, String),
    #[error("Invalid messages count: referenced ({0}) != no transaction ({1}) + conflicting ({2}) + included ({3})")]
    InvalidMessagesCount(usize, usize, usize, usize),
    #[error("Invalid ledger unspent state: {0}")]
    InvalidLedgerUnspentState(u64),
    #[error("Invalid ledger balance state: {0}")]
    InvalidLedgerBalanceState(u64),
    #[error("Invalid ledger dust state: {0:?} {1:?}")]
    InvalidLedgerDustState(Address, Balance),
    #[error("Consumed amount overflow: {0}.")]
    ConsumedAmountOverflow(u128),
    #[error("Created amount overflow: {0}.")]
    CreatedAmountOverflow(u128),
    #[error("Ledger state overflow: {0}")]
    LedgerStateOverflow(u128),
    #[error("Non zero balance diff sum: {0}.")]
    NonZeroBalanceDiffSum(i64),
    #[error("Decreasing receipt migrated at index: {0} < {1}")]
    DecreasingReceiptMigratedAtIndex(MilestoneIndex, MilestoneIndex),
    #[error("Missing unspent output {0}")]
    MissingUnspentOutput(Unspent),
    #[error("Storage backend error: {0}")]
    Storage(Box<dyn std::error::Error + Send>),
}
