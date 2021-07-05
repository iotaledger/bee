// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the errors that can occur during ledger operations.

use crate::{
    types::{Balance, Error as TypesError, Unspent},
    workers::snapshot::error::Error as SnapshotError,
};

use bee_message::{address::Address, milestone::MilestoneIndex, Error as MessageError, MessageId};

/// Errors occurring during ledger workers operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Snapshot error.
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),
    /// Types error.
    #[error("Types error: {0}")]
    Types(#[from] TypesError),
    /// Message error.
    #[error("Message error: {0}")]
    Message(#[from] MessageError),
    /// Missing message in the past cone of the milestone
    #[error("Message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    /// Unsupported input kind.
    #[error("Unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    /// Unsupported output kind.
    #[error("Unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    /// Unsupported payload kind.
    #[error("Unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    /// Milestone message not found.
    #[error("Milestone message not found: {0}")]
    MilestoneMessageNotFound(MessageId),
    /// Message payload is not a milestone
    #[error("Message payload is not a milestone")]
    NoMilestonePayload,
    /// Non contiguous milestones.
    #[error("Non contiguous milestones: tried to confirm {0} on top of {1}")]
    NonContiguousMilestones(u32, u32),
    /// Merkle proof mismatch.
    #[error("Merkle proof mismatch on milestone {0}: computed {1} != provided {2}")]
    MerkleProofMismatch(MilestoneIndex, String, String),
    /// Invalid messages count.
    #[error("Invalid messages count: referenced ({0}) != no transaction ({1}) + conflicting ({2}) + included ({3})")]
    InvalidMessagesCount(usize, usize, usize, usize),
    /// Invalid ledger unspent state.
    #[error("Invalid ledger unspent state: {0}")]
    InvalidLedgerUnspentState(u64),
    /// Invalid ledger balance state.
    #[error("Invalid ledger balance state: {0}")]
    InvalidLedgerBalanceState(u64),
    /// Invalid ledger dust state.
    #[error("Invalid ledger dust state: {0:?} {1:?}")]
    InvalidLedgerDustState(Address, Balance),
    /// Consumed amount overflow.
    #[error("Consumed amount overflow: {0}.")]
    ConsumedAmountOverflow(u128),
    /// Created amount overflow.
    #[error("Created amount overflow: {0}.")]
    CreatedAmountOverflow(u128),
    /// Ledger state overflow.
    #[error("Ledger state overflow: {0}")]
    LedgerStateOverflow(u128),
    /// Non zero balance diff sum.
    #[error("Non zero balance diff sum: {0}.")]
    NonZeroBalanceDiffSum(i64),
    /// Decreasing receipt migrated at index.
    #[error("Decreasing receipt migrated at index: {0} < {1}")]
    DecreasingReceiptMigratedAtIndex(MilestoneIndex, MilestoneIndex),
    /// Missing unspent output.
    #[error("Missing unspent output {0}")]
    MissingUnspentOutput(Unspent),
    /// Storage backend error.
    #[error("Storage backend error: {0}")]
    Storage(Box<dyn std::error::Error + Send>),
}
