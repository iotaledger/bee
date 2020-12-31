// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::Error as MessageError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error happened: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid variant read")]
    InvalidVariant,
    #[error("Invalid version read: {0}, {1}")]
    InvalidVersion(u8, u8),
    #[error("")]
    NoDownloadSourceAvailable,
    #[error("")]
    InvalidFilePath(String),
    #[error("{0}")]
    Message(#[from] MessageError),
    #[error("Network Id mismatch: configuration {0} != snapshot {1}")]
    NetworkIdMismatch(u64, u64),
    #[error("")]
    LedgerSepIndexesInconsistency(u32, u32),
    #[error("")]
    InvalidMilestoneDiffsCount(usize, usize),
    #[error(
        "Only a delta snapshot file exists, without a full snapshot file. Remove the delta snapshot file and restart"
    )]
    OnlyDeltaFileExists,
    #[error("Storage operation failed: {0}")]
    StorageBackend(Box<dyn std::error::Error>),
}
