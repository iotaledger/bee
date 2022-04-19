// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::{storage::StorageBackend, Tangle};

use crate::{types::LedgerIndex, workers::snapshot::config::SnapshotConfig};

/// Reasons for skipping snapshotting.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SnapshottingSkipReason {
    #[error("Snapshotting skipped for the next {reached_in} indexes.")]
    BelowThreshold { reached_in: u32 },
    #[error("Snapshotting deferred for the next {next_in} indexes.")]
    Deferred { next_in: u32 },
}

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &Tangle<B>,
    ledger_index: LedgerIndex,
    snapshot_depth: u32,
    snapshot_config: &SnapshotConfig,
) -> Result<(), SnapshottingSkipReason> {
    let snapshot_index = *tangle.get_snapshot_index();

    let snapshot_interval = if tangle.is_synced() {
        snapshot_config.interval_synced()
    } else {
        snapshot_config.interval_unsynced()
    };

    if *ledger_index < snapshot_depth {
        Err(SnapshottingSkipReason::BelowThreshold {
            reached_in: snapshot_depth - *ledger_index,
        })
    } else if *ledger_index < snapshot_index + snapshot_interval {
        Err(SnapshottingSkipReason::Deferred {
            next_in: (snapshot_index + snapshot_interval) - *ledger_index,
        })
    } else {
        Ok(())
    }
}
