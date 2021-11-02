// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{types::LedgerIndex, workers::snapshot::config::SnapshotConfig};

use bee_tangle::{storage::StorageBackend, Tangle};

/// Reasons for skipping snapshotting.
#[derive(Debug)]
pub enum SnapshottingSkipReason {
    /// Not enough data yet to create a snapshot.
    BelowThreshold { reached_in: u32 },
    /// Snapshotting is deferred to a later milestone.
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
