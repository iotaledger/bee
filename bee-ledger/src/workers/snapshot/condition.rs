// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_tangle::{storage::StorageBackend, Tangle};

use crate::{types::LedgerIndex, workers::snapshot::config::SnapshotConfig};

/// Reasons for skipping snapshotting.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SnapshottingSkipReason {
    #[error("disabled")]
    Disabled,
    #[error("ledger index < snapshotting depth")]
    BelowDepth,
    #[error("ledger index < next snapshot index {next_snapshot_index}")]
    BelowNextSnapshotIndex { next_snapshot_index: u32 },
}

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &Tangle<B>,
    ledger_index: LedgerIndex,
    snapshot_depth: u32,
    snapshot_config: &SnapshotConfig,
) -> Result<(), SnapshottingSkipReason> {
    if !snapshot_config.snapshotting().enabled() {
        return Err(SnapshottingSkipReason::Disabled);
    }

    // Get the index of the last snapshot.
    let snapshot_index = *tangle.get_snapshot_index();

    let snapshot_interval = if tangle.is_synced() {
        snapshot_config.snapshotting().interval_synced()
    } else {
        snapshot_config.snapshotting().interval_unsynced()
    };

    if *ledger_index < snapshot_index + snapshot_depth {
        Err(SnapshottingSkipReason::BelowDepth)
    } else if *ledger_index < snapshot_index + snapshot_interval {
        Err(SnapshottingSkipReason::BelowNextSnapshotIndex {
            next_snapshot_index: snapshot_index + snapshot_interval,
        })
    } else {
        Ok(())
    }
}
