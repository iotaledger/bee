// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{types::LedgerIndex, workers::snapshot::config::SnapshotConfig};

use bee_tangle::{storage::StorageBackend, MsTangle};

#[derive(Debug)]
pub enum SkipReason {
    /// There is not enough data yet to create a snapshot.
    History { available_in: u32 },

    /// Snapshotting is deferred to a later milestone.
    Interval { next_in: u32 },
}

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_index: LedgerIndex,
    pruning_depth_min: u32,
    snapshot_config: &SnapshotConfig,
) -> Result<(), SkipReason> {
    // FIXME: `tangle` has not setter for the snapshot index, so it cannot be updated currently.
    let snapshot_index = *tangle.get_snapshot_index();

    let snapshot_interval = if tangle.is_synced() {
        snapshot_config.interval_synced()
    } else {
        snapshot_config.interval_unsynced()
    };

    if *ledger_index < pruning_depth_min {
        // Not enough history.
        Err(SkipReason::History {
            available_in: pruning_depth_min - *ledger_index,
        })
    } else if *ledger_index < snapshot_index + snapshot_interval {
        // Not now.
        Err(SkipReason::Interval {
            next_in: (snapshot_index + snapshot_interval) - *ledger_index,
        })
    } else {
        Ok(())
    }
}
