// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{snapshot::SnapshotKind, Error as TypesError};

use bee_message::milestone::MilestoneIndex;

use thiserror::Error;

/// Errors occurring during snapshot operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error happened.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Types error.
    #[error("types error: {0}")]
    Types(#[from] TypesError),
    /// Unexpected snapshot kind.
    #[error("unexpected snapshot kind: expected {0:?}, read {1:?}")]
    UnexpectedSnapshotKind(SnapshotKind, SnapshotKind),
    /// Downloading failed.
    #[error("downloading failed")]
    DownloadingFailed,
    /// No snapshot download source available.
    #[error("no snapshot download source available")]
    NoDownloadSourceAvailable,
    /// Invalid file path.
    #[error("invalid file path: {0}")]
    InvalidFilePath(String),
    /// Network id mismatch between configuration and snapshot.
    #[error("network id mismatch between configuration and snapshot: {0} != {1}")]
    NetworkIdMismatch(u64, u64),
    /// Inconsistency between ledger index and sep index.
    #[error("inconsistency between ledger index {0} and sep index {1}")]
    LedgerSepIndexesInconsistency(MilestoneIndex, MilestoneIndex),
    /// Invalid milestone diffs count.
    #[error("invalid milestone diffs count: expected {0}, read {1}")]
    InvalidMilestoneDiffsCount(usize, usize),
    /// Only a delta snapshot file exists without a full snapshot file.
    #[error(
        "only a delta snapshot file exists without a full snapshot file (remove the delta snapshot file and restart)"
    )]
    OnlyDeltaSnapshotFileExists,
    /// Unexpected milestone diff index.
    #[error("unexpected milestone diff index: {0:?}")]
    UnexpectedMilestoneDiffIndex(MilestoneIndex),
    /// Missing consumed treasury.
    #[error("missing consumed treasury")]
    MissingConsumedTreasury,
    /// Remaining bytes in file.
    #[error("remaining bytes in file")]
    RemainingBytes,
}
