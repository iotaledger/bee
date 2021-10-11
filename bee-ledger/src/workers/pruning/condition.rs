// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{types::LedgerIndex, workers::pruning::config::PruningConfig};

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, Tangle};

const PRUNING_BATCH_SIZE_MAX: u32 = 200;

/// Reasons for skipping pruning.
#[derive(Debug)]
pub enum PruningSkipReason {
    /// Pruning is disabled in the config.
    Disabled,
    /// Not enough data yet to be pruned.
    BelowThreshold { reached_in: u32 },
}

pub(crate) fn should_prune<B: StorageBackend>(
    tangle: &Tangle<B>,
    ledger_index: LedgerIndex,
    pruning_delay: u32,
    config: &PruningConfig,
) -> Result<(MilestoneIndex, MilestoneIndex), PruningSkipReason> {
    if config.disabled() {
        return Err(PruningSkipReason::Disabled);
    }

    let pruning_index = *tangle.get_pruning_index() + 1;
    let pruning_threshold = pruning_index + pruning_delay;

    if *ledger_index < pruning_threshold {
        Err(PruningSkipReason::BelowThreshold {
            reached_in: pruning_threshold - *ledger_index,
        })
    } else {
        let target_pruning_index = *ledger_index - pruning_delay;

        Ok((
            pruning_index.into(),
            if target_pruning_index > pruning_index + PRUNING_BATCH_SIZE_MAX {
                (pruning_index + PRUNING_BATCH_SIZE_MAX).into()
            } else {
                target_pruning_index.into()
            },
        ))
    }
}
