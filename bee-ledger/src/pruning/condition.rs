// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::pruning::config::PruningConfig;

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, MsTangle};

pub(crate) fn should_prune<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    cmi: MilestoneIndex,
    delay: u32,
    config: &PruningConfig,
) -> Option<MilestoneIndex> {
    // Do not prune if pruning is disabled in the config.
    if !config.enabled() {
        return None;
    }

    // Do not prune if there isn't old enough data to prune yet. This will only happen for a freshly started node.
    if *cmi < delay {
        return None;
    }

    // Do not prune out of interval.
    if *cmi % config.interval() != 0 {
        return None;
    }

    // Return the `target_index`, i.e. the `MilestoneIndex` up to which the database can be savely pruned.
    Some(MilestoneIndex(*cmi - delay))
}
