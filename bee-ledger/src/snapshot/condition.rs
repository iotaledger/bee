// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{consensus::StorageBackend, snapshot::config::SnapshotConfig};

use bee_message::milestone::MilestoneIndex;
use bee_tangle::MsTangle;

pub(crate) const SNAPSHOT_DEPTH_MIN: u32 = 5;

pub(crate) fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    index: MilestoneIndex,
    depth: u32,
    config: &SnapshotConfig,
) -> Option<MilestoneIndex> {
    let current_solid_index = *index;

    let snapshot_index = *tangle.get_snapshot_index();
    debug_assert!(current_solid_index > snapshot_index);

    // If the node is unsync we snapshot less often.
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    // Do not snapshot without enough depth. This will only happen for a freshly started node.
    if current_solid_index < depth {
        return None;
    }

    // Do not snapshot out of interval.
    if current_solid_index % snapshot_interval != 0 {
        return None;
    }

    Some(MilestoneIndex(current_solid_index - depth))
}
