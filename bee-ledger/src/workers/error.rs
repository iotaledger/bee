// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the errors that can occur during ledger operations.

use std::convert::Infallible;

use bee_message::{payload::milestone::MilestoneIndex, Error as MessageError, MessageId};
use packable::error::UnpackError;

use crate::{
    types::{Error as TypesError, Unspent},
    workers::snapshot::error::Error as SnapshotError,
};

/// Errors occurring during ledger workers operations.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("snapshot error: {0}")]
    Snapshot(#[from] SnapshotError),
    #[error("types error: {0}")]
    Types(#[from] TypesError),
    #[error("message error: {0}")]
    Message(#[from] MessageError),
    #[error("message {0} is missing in the past cone of the milestone")]
    MissingMessage(MessageId),
    #[error("unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    #[error("unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    #[error("unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    #[error("milestone message not found: {0}")]
    MilestoneMessageNotFound(MessageId),
    #[error("message payload is not a milestone")]
    NoMilestonePayload,
    #[error("non contiguous milestones: tried to confirm {0} on top of {1}")]
    NonContiguousMilestones(u32, u32),
    #[error("confirmed merkle root mismatch on milestone {0}: computed {1} != provided {2}")]
    ConfirmedMerkleRootMismatch(MilestoneIndex, String, String),
    #[error("applied merkle root mismatch on milestone {0}: computed {1} != provided {2}")]
    AppliedMerkleRootMismatch(MilestoneIndex, String, String),
    #[error("invalid messages count: referenced ({0}) != no transaction ({1}) + conflicting ({2}) + included ({3})")]
    InvalidMessagesCount(usize, usize, usize, usize),
    #[error("invalid ledger unspent state: {0}")]
    InvalidLedgerUnspentState(u64),
    #[error("consumed amount overflow")]
    ConsumedAmountOverflow,
    #[error("created amount overflow")]
    CreatedAmountOverflow,
    #[error("consumed native tokens amount overflow")]
    ConsumedNativeTokensAmountOverflow,
    #[error("created native tokens amount overflow")]
    CreatedNativeTokensAmountOverflow,
    #[error("ledger state overflow: {0}")]
    LedgerStateOverflow(u128),
    #[error("decreasing receipt migrated at index: {0} < {1}")]
    DecreasingReceiptMigratedAtIndex(MilestoneIndex, MilestoneIndex),
    #[error("missing unspent output {0}")]
    MissingUnspentOutput(Unspent),
    #[error("storage backend error: {0}")]
    Storage(Box<dyn std::error::Error + Send>),
    #[error("storage deposit return overflow")]
    StorageDepositReturnOverflow,
    #[error("previous milestone not found in the past cone")]
    PreviousMilestoneNotFound,
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
