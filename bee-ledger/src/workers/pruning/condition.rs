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
    NotEnoughData,

    /// Pruning is skipped for the current confirmed milestone.
    Delayed,
}

pub(crate) fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    ledger_index: LedgerIndex,
    pruning_config: &PruningConfig,
) -> Result<PruningTargetIndex, SkipReason> {
    if pruning_config.disabled() {
        Err(SkipReason::Disabled)
    } else if *ledger_index < *tangle.get_pruning_index() + pruning_config.delay() {
        Err(SkipReason::NotEnoughData)
    } else if *ledger_index % pruning_config.interval() != 0 {
        Err(SkipReason::Delayed)
    } else {
        // Subtracting unsigned:
        // We made sure that `confirmed_milestone_index` >= pruning_config.delay() always holds.
        let pruning_target_index = *ledger_index - pruning_config.delay();

        Ok(pruning_target_index.into())
    }
}
