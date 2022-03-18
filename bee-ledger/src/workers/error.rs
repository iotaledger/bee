// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the errors that can occur during ledger operations.

use crate::{
    types::{Error as TypesError, Unspent},
    workers::snapshot::error::Error as SnapshotError,
};

use bee_message::{milestone::MilestoneIndex, Error as MessageError, MessageId};

use packable::error::UnpackError;

use std::convert::Infallible;

/// Errors occurring during ledger workers operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Snapshot error.
    #[error("snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),
    /// Types error.
    #[error("types error: {0}")]
    Types(#[from] TypesError),
    /// Message error.
    #[error("message error: {0}")]
    Message(#[from] MessageError),
    /// Missing message in the past cone of the milestone
    #[error("message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    /// Unsupported input kind.
    #[error("unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    /// Unsupported output kind.
    #[error("unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    /// Unsupported payload kind.
    #[error("unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    /// Milestone message not found.
    #[error("milestone message not found: {0}")]
    MilestoneMessageNotFound(MessageId),
    /// Message payload is not a milestone
    #[error("message payload is not a milestone")]
    NoMilestonePayload,
    /// Non contiguous milestones.
    #[error("non contiguous milestones: tried to confirm {0} on top of {1}")]
    NonContiguousMilestones(u32, u32),
    /// Merkle proof mismatch.
    #[error("merkle proof mismatch on milestone {0}: computed {1} != provided {2}")]
    MerkleProofMismatch(MilestoneIndex, String, String),
    /// Invalid messages count.
    #[error("invalid messages count: referenced ({0}) != no transaction ({1}) + conflicting ({2}) + included ({3})")]
    InvalidMessagesCount(usize, usize, usize, usize),
    /// Invalid ledger unspent state.
    #[error("invalid ledger unspent state: {0}")]
    InvalidLedgerUnspentState(u64),
    /// Consumed amount overflow.
    #[error("consumed amount overflow")]
    ConsumedAmountOverflow,
    /// Created amount overflow.
    #[error("created amount overflow.")]
    CreatedAmountOverflow,
    /// Consumed native tokens amount overflow.
    #[error("consumed native tokens amount overflow.")]
    ConsumedNativeTokensAmountOverflow,
    /// Created native tokens amount overflow.
    #[error("created native tokens amount overflow.")]
    CreatedNativeTokensAmountOverflow,
    /// Ledger state overflow.
    #[error("ledger state overflow: {0}")]
    LedgerStateOverflow(u128),
    /// Decreasing receipt migrated at index.
    #[error("decreasing receipt migrated at index: {0} < {1}")]
    DecreasingReceiptMigratedAtIndex(MilestoneIndex, MilestoneIndex),
    /// Missing unspent output.
    #[error("missing unspent output {0}")]
    MissingUnspentOutput(Unspent),
    /// Storage backend error.
    #[error("storage backend error: {0}")]
    Storage(Box<dyn std::error::Error + Send>),
    /// Storage deposit return overflow.
    #[error("storage deposit return overflow")]
    StorageDepositReturnOverflow,
}

impl<E: Into<Error>> From<UnpackError<E, std::io::Error>> for Error {
    fn from(err: UnpackError<E, std::io::Error>) -> Self {
        match err {
            UnpackError::Packable(err) => err.into(),
            UnpackError::Unpacker(err) => err.into(),
        }
    }
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
