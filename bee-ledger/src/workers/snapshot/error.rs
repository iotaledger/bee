// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::milestone::MilestoneIndex;
use thiserror::Error;

use crate::types::{snapshot::SnapshotKind, Error as TypesError};

/// Errors occurring during snapshot operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("downloading failed")]
    DownloadingFailed,
    #[error("invalid file path: {0}")]
    InvalidFilePath(String),
    #[error("invalid milestone diffs count: expected {0}, read {1}")]
    InvalidMilestoneDiffsCount(usize, usize),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("inconsistency between ledger index {0} and sep index {1}")]
    LedgerSepIndexesInconsistency(MilestoneIndex, MilestoneIndex),
    #[error("missing consumed treasury")]
    MissingConsumedTreasury,
    #[error("network id mismatch between configuration and snapshot: {0} != {1}")]
    NetworkIdMismatch(u64, u64),
    #[error("no snapshot download source available")]
    NoDownloadSourceAvailable,
    #[error(
        "only a delta snapshot file exists without a full snapshot file (remove the delta snapshot file and restart)"
    )]
    OnlyDeltaSnapshotFileExists,
    #[error("parsing snapshot header failed: {0}")]
    ParsingSnapshotHeaderFailed(TypesError),
    #[error("remaining bytes in file")]
    RemainingBytes,
    #[error("types error: {0}")]
    Types(#[from] TypesError),
    #[error("unexpected snapshot kind: expected {0:?}, read {1:?}")]
    UnexpectedSnapshotKind(SnapshotKind, SnapshotKind),
    #[error("unexpected milestone diff index: {0:?}")]
    UnexpectedMilestoneDiffIndex(MilestoneIndex),
}
