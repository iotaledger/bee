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

    /// There is not enough data yet to be pruned according to the config settings.
    Unnecessary { necessary_in: u32 },

    /// Pruning is deferred to a later milestone.
    Deferred { next_pruning_in: u32 },
}

pub(crate) fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_index: LedgerIndex,
    pruning_config: &PruningConfig,
) -> Result<PruningTargetIndex, SkipReason> {
    let start_index_min = || *tangle.get_pruning_index() + pruning_config.delay() + 1;

    if pruning_config.disabled() {
        Err(SkipReason::Disabled)
    } else if *ledger_index < start_index_min() {
        Err(SkipReason::Unnecessary {
            necessary_in: start_index_min() - *ledger_index,
        })
    } else if *ledger_index % pruning_config.interval() != 0 {
        Err(SkipReason::Deferred {
            next_pruning_in: pruning_config.interval() - (*ledger_index % pruning_config.interval()),
        })
    } else {
        // Subtracting unsigned:
        // We made sure that `confirmed_milestone_index` >= pruning_config.delay() always holds.
        let pruning_target_index = *ledger_index - pruning_config.delay();

        Ok(pruning_target_index.into())
    }
}
