// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, Tangle};

use crate::{types::LedgerIndex, workers::pruning::config::PruningConfig};

const PRUNING_BATCH_SIZE_MAX: u32 = 200;

/// Reasons for skipping pruning.
#[derive(Debug)]
pub(crate) enum PruningSkipReason {
    /// Pruning is disabled in the config.
    Disabled,
    /// The storage in use doesn't support this pruning task.
    PruningTaskNotSupported,
    /// Not enough history yet to be pruned.
    BelowMaxMilestonesToKeepThreshold { reached_in: u32 },
    /// The database has not yet reached its target size.
    BelowTargetSize { reached_in: usize },
    /// The cooldown time has not yet been passed.
    BelowCooldownTimeThreshold { reached_in: Duration },
}

pub(crate) enum PruningTask {
    // TODO: consider using named structs: start index, target index
    ByRange(MilestoneIndex, MilestoneIndex),
    // TODO: same: reduced size
    BySize(usize),
}

pub(crate) fn should_prune<S: StorageBackend>(
    tangle: &Tangle<S>,
    storage: &S,
    ledger_index: LedgerIndex,
    milestones_to_keep: u32,
    config: &PruningConfig,
) -> Result<PruningTask, PruningSkipReason> {
    if config.milestones().enabled() {
        let pruning_index = *tangle.get_pruning_index() + 1;
        let pruning_threshold = pruning_index + milestones_to_keep;

        if *ledger_index < pruning_threshold {
            Err(PruningSkipReason::BelowMaxMilestonesToKeepThreshold {
                reached_in: pruning_threshold - *ledger_index,
            })
        } else {
            let target_pruning_index = *ledger_index - milestones_to_keep;

            Ok(PruningTask::ByRange(
                pruning_index.into(),
                if target_pruning_index > pruning_index + PRUNING_BATCH_SIZE_MAX {
                    (pruning_index + PRUNING_BATCH_SIZE_MAX).into()
                } else {
                    target_pruning_index.into()
                },
            ))
        }
    } else if config.size().enabled() {
        // TODO: return `PruningTAskNotSupported` if we can't get the size of the storage.
        let actual_size = storage.size().expect("ok storage size").expect("some storage size");
        let target_size = config.size().target_size();

        log::debug!("Storage size: actual {actual_size} target {target_size}");

        if actual_size < target_size {
            // Panic: cannot underflow.
            Err(PruningSkipReason::BelowTargetSize {
                reached_in: target_size - actual_size,
            })
        } else {
            let reduced_size =
                actual_size - (config.size().threshold_percentage() as f64 / 100.0 * target_size as f64) as usize;

            log::debug!("Reduced size: {reduced_size}");

            Ok(PruningTask::BySize(reduced_size))
        }
    } else {
        Err(PruningSkipReason::Disabled)
    }
}
