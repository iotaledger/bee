// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::config::PruningConfig;

use crate::types::LedgerIndex;

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, MsTangle};

type PruningTargetIndex = MilestoneIndex;

/// Reasons for not-pruning.
#[derive(Debug)]
pub enum SkipReason {
    /// Pruning is disabled in the config.
    Disabled,

    /// There is not enough data yet to be pruned.
    BelowThreshold { reached_in: u32 },
}

pub(crate) fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_index: LedgerIndex,
    pruning_depth: u32,
    pruning_config: &PruningConfig,
) -> Result<PruningTargetIndex, SkipReason> {
    let pruning_index = *tangle.get_pruning_index();
    let start_index_min = || pruning_index + pruning_depth + 1;

    if pruning_config.disabled() {
        // Disabled in the config.
        Err(SkipReason::Disabled)
    } else if *ledger_index < start_index_min() {
        // Not enough history yet.
        Err(SkipReason::BelowThreshold {
            reached_in: start_index_min() - *ledger_index,
        })
    } else {
        // NB:
        // We made sure that `confirmed_milestone_index` >= pruning_depth.
        let pruning_target_index = *ledger_index - pruning_depth;

        Ok(pruning_target_index.into())
    }
}
