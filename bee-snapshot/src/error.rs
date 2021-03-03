// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::kind::Kind;

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
    #[error("Unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    #[error("")]
    NoDownloadSourceAvailable,
    #[error("")]
    InvalidFilePath(String),
    #[error("{0}")]
    Message(#[from] MessageError),
    #[error("Network Id mismatch: configuration {0} != snapshot {1}")]
    NetworkIdMismatch(u64, u64),
    #[error("")]
    LedgerSepIndexesInconsistency(MilestoneIndex, MilestoneIndex),
    #[error("")]
    InvalidMilestoneDiffsCount(usize, usize),
    #[error(
        "Only a delta snapshot file exists, without a full snapshot file. Remove the delta snapshot file and restart"
    )]
    OnlyDeltaFileExists,
    #[error("Storage operation failed: {0}")]
    StorageBackend(Box<dyn std::error::Error + Send + 'static>),
    #[error("")]
    InvalidPayloadKind,
}
