// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    pruning::{
        config::PruningConfig,
        constants::{PRUNING_THRESHOLD, SOLID_ENTRY_POINT_THRESHOLD_PAST},
    },
    snapshot::config::SnapshotConfig,
};

use bee_message::milestone::MilestoneIndex;

use bee_tangle::{storage::StorageBackend, MsTangle};

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    index: MilestoneIndex,
    depth: u32,
    config: &SnapshotConfig,
) -> bool {
    let solid_index = *index;
    let snapshot_index = *tangle.get_snapshot_index();
    let pruning_index = *tangle.get_pruning_index();
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    if (solid_index < depth + snapshot_interval)
        || (solid_index - depth) < pruning_index + 1 + SOLID_ENTRY_POINT_THRESHOLD_PAST
    {
        // Not enough history to calculate solid entry points.
        return false;
    }

    solid_index - (depth + snapshot_interval) >= snapshot_index
}

pub(crate) fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    mut index: MilestoneIndex,
    delay: u32,
    config: &PruningConfig,
) -> bool {
    if !config.enabled() {
        return false;
    }

    if *index <= delay {
        return false;
    }

    // Pruning happens after creating the snapshot so the metadata should provide the latest index.
    if *tangle.get_snapshot_index() < SOLID_ENTRY_POINT_THRESHOLD_PAST + PRUNING_THRESHOLD + 1 {
        return false;
    }

    let target_index_max =
        MilestoneIndex(*tangle.get_snapshot_index() - SOLID_ENTRY_POINT_THRESHOLD_PAST - PRUNING_THRESHOLD - 1);

    if index > target_index_max {
        index = target_index_max;
    }

    if tangle.get_pruning_index() >= index {
        return false;
    }

    // We prune in "PRUNING_THRESHOLD" steps to recalculate the solid_entry_points.
    if *tangle.get_entry_point_index() + PRUNING_THRESHOLD + 1 > *index {
        return false;
    }

    true
}
