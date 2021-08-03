// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::config::PruningConfig;

use crate::types::LedgerIndex;

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, MsTangle};

const MAX_PRUNING_BATCH_SIZE: u32 = 200;

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
    delay: u32,
    config: &PruningConfig,
) -> Result<(MilestoneIndex, MilestoneIndex), SkipReason> {
    let start_index = *tangle.get_pruning_index() + 1;
    let ledger_index_min = start_index + delay;

    if config.disabled() {
        // Disabled in the config.
        Err(SkipReason::Disabled)
    } else if *ledger_index < ledger_index_min {
        // Not enough history yet.
        Err(SkipReason::BelowThreshold {
            reached_in: ledger_index_min - *ledger_index,
        })
    } else {
        // Note: we made sure that `ledger_index` >= delay.
        let target_index = *ledger_index - delay;

        if target_index > start_index + MAX_PRUNING_BATCH_SIZE {
            Ok((start_index.into(), (start_index + MAX_PRUNING_BATCH_SIZE).into()))
        } else {
            Ok((start_index.into(), target_index.into()))
        }
    }
}
