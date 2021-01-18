// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::kind::Kind;

use bee_ledger::model::Error as LedgerError;
use bee_message::{milestone::MilestoneIndex, Error as MessageError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error happened: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid variant read")]
    InvalidVariant,
    #[error("Invalid snapshot version: node supports {0}, read {1}")]
    InvalidVersion(u8, u8),
    #[error("Invalid snapshot kind: expected {0:?}, read {1:?}")]
    InvalidKind(Kind, Kind),
    #[error("")]
    NoDownloadSourceAvailable,
    #[error("")]
    InvalidFilePath(String),
    #[error("{0}")]
    Message(#[from] MessageError),
    #[error("{0}")]
    Ledger(#[from] LedgerError),
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
    #[error("Unexpected milestine diff index: {0:?}.")]
    UnexpectedDiffIndex(MilestoneIndex),
    #[error("Invalid ledger state.")]
    InvalidLedgerState,
    #[error("Storage operation failed: {0}")]
    StorageBackend(Box<dyn std::error::Error>),
}
