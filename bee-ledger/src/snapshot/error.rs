// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{snapshot::kind::Kind, types::Error as TypeError};

use bee_message::{milestone::MilestoneIndex, Error as MessageError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error happened: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid snapshot kind: {0}")]
    InvalidKind(u8),
    #[error("Unsupported snapshot version: supports {0}, read {1}")]
    UnsupportedVersion(u8, u8),
    #[error("Unexpected snapshot kind: expected {0:?}, read {1:?}")]
    UnexpectedKind(Kind, Kind),
    #[error("No snapshot download source available")]
    NoDownloadSourceAvailable,
    #[error("Invalid snapshot path: {0}")]
    InvalidFilePath(String),
    #[error("Message error: {0}")]
    Message(#[from] MessageError),
    #[error("Type error: {0}")]
    Type(#[from] TypeError),
    #[error("Network id mismatch between configuration and snapshot: {0} != {1}")]
    NetworkIdMismatch(u64, u64),
    #[error("Inconsistency between ledger index {0} and sep index {1}")]
    LedgerSepIndexesInconsistency(MilestoneIndex, MilestoneIndex),
    #[error("Invalid milestone diffs count: expected {0}, read {1}")]
    InvalidMilestoneDiffsCount(usize, usize),
    #[error("Invalid payload kind: {0}")]
    InvalidPayloadKind(u32),
    #[error("")]
    UnsupportedOutputKind(u8),
    #[error(
        "Only a delta snapshot file exists, without a full snapshot file. Remove the delta snapshot file and restart"
    )]
    OnlyDeltaSnapshotFileExists,
    #[error("Unexpected milestine diff index: {0:?}")]
    UnexpectedDiffIndex(MilestoneIndex),
    #[error("Storage operation failed: {0}")]
    StorageBackend(Box<dyn std::error::Error + Send + 'static>),
    #[error("")]
    Consumer(Box<dyn std::error::Error + Send + 'static>),
}
