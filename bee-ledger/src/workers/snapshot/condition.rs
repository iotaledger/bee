// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::config::SnapshotConfig;

use crate::{types::LedgerIndex, workers::StorageBackend};

use bee_message::milestone::MilestoneIndex;
use bee_tangle::MsTangle;

pub(crate) const SNAPSHOT_DEPTH_MIN: u32 = 5;

type SnapshotIndex = MilestoneIndex;

/// Reasons for not doing a snapshot.
#[derive(Debug)]
pub enum SkipReason {
    /// Snapshotting is disabled in the config.
    Disabled,

    /// There is not enough data yet to do a snapshot.
    NotEnoughData,

    /// Snapshotting is skipped for the current confirmed milestone.
    NotNecessary,
}

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_index: LedgerIndex,
    snapshot_config: &SnapshotConfig,
) -> Result<SnapshotIndex, SkipReason> {
    // TODO: allow snapshotting to be disabled

    let snapshot_index = tangle.get_snapshot_index();

    // Panic:
    // If this invariant doesn't hold, then something really shady is going on, and we should halt the node
    // immediatedly.
    assert!(*ledger_index > *snapshot_index, "Time went backwards?");

    // If the node is unsync we snapshot less often.
    let snapshot_interval = if tangle.is_synced() {
        snapshot_config.interval_synced()
    } else {
        snapshot_config.interval_unsynced()
    };

    if *ledger_index < snapshot_config.depth() {
        Err(SkipReason::NotEnoughData)
    } else if *ledger_index < *snapshot_index + snapshot_interval {
        Err(SkipReason::NotNecessary)
    } else {
        // Subtracting unsigned:
        // We made sure that `confirmed_milestone_index` >= snapshot_config.depth() always holds.
        let snapshot_index = *ledger_index - snapshot_config.depth();

        Ok(snapshot_index.into())
    }
}
