// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::snapshot::SnapshotKind;

use bee_message::milestone::MilestoneIndex;

use thiserror::Error;

/// Errors occurring during snapshot operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error happened.
    #[error("I/O error happened: {0}")]
    Io(#[from] std::io::Error),
    /// Unexpected snapshot kind.
    #[error("Unexpected snapshot kind: expected {0:?}, read {1:?}")]
    UnexpectedSnapshotKind(SnapshotKind, SnapshotKind),
    /// Downloading failed.
    #[error("Downloading failed.")]
    DownloadingFailed,
    /// No snapshot download source available.
    #[error("No snapshot download source available")]
    NoDownloadSourceAvailable,
    /// Invalid file path.
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
    /// Network id mismatch between configuration and snapshot.
    #[error("Network id mismatch between configuration and snapshot: {0} != {1}")]
    NetworkIdMismatch(u64, u64),
    /// Inconsistency between ledger index and sep index.
    #[error("Inconsistency between ledger index {0} and sep index {1}")]
    LedgerSepIndexesInconsistency(MilestoneIndex, MilestoneIndex),
    /// Invalid milestone diffs count.
    #[error("Invalid milestone diffs count: expected {0}, read {1}")]
    InvalidMilestoneDiffsCount(usize, usize),
    /// Only a delta snapshot file exists without a full snapshot file.
    #[error(
        "Only a delta snapshot file exists without a full snapshot file. Remove the delta snapshot file and restart"
    )]
    OnlyDeltaSnapshotFileExists,
    /// Unexpected milestone diff index.
    #[error("Unexpected milestone diff index: {0:?}")]
    UnexpectedMilestoneDiffIndex(MilestoneIndex),
    /// Missing consumed treasury.
    #[error("Missing consumed treasury.")]
    MissingConsumedTreasury,
    /// Remaining bytes in file.
    #[error("Remaining bytes in file.")]
    RemainingBytes,
}
