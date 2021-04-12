// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::pruning::config::PruningConfig;

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, MsTangle};

pub(crate) const PRUNING_INTERVAL: u32 = 50;

pub(crate) fn should_prune<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    index: MilestoneIndex,
    delay: u32,
    config: &PruningConfig,
) -> Option<MilestoneIndex> {
    // Do not prune if pruning is disabled in the config.
    if !config.enabled() {
        return None;
    }

    let current_solid_index = *index;

    // Do not prune if there isn't old enough data to prune yet. This will only happen for a freshly started node.
    if current_solid_index < delay {
        return None;
    }

    // Do not prune out of interval.
    if current_solid_index % PRUNING_INTERVAL != 0 {
        return None;
    }

    // Return the `target_index`, i.e. the `MilestoneIndex` up to which the database can be savely pruned.
    Some(MilestoneIndex(current_solid_index - delay))
}
