// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// use std::time::Duration;

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bee_message::milestone::MilestoneIndex;
use bee_tangle::{storage::StorageBackend, Tangle};

use crate::{types::LedgerIndex, workers::pruning::config::PruningConfig};

const PRUNING_BATCH_SIZE_MAX: u32 = 200;

static PREVIOUS_PRUNING_BY_SIZE: AtomicU64 = AtomicU64::new(0);

/// Reasons for skipping pruning.
#[derive(Debug, thiserror::Error)]
pub(crate) enum PruningSkipReason {
    #[error("disabled")]
    Disabled,
    #[error("ledger index < target index threshold")]
    BelowTargetIndexThreshold,
    #[error("size metric not supported by the storage layer")]
    SizeMetricUnsupported,
    #[error("size metric currently unavailable")]
    SizeMetricUnavailable,
    #[error("current size < target size threshold")]
    BelowTargetSizeThreshold,
    #[error("cooldown")]
    Cooldown,
}

pub(crate) enum PruningTask {
    ByIndexRange {
        start_index: MilestoneIndex,
        target_index: MilestoneIndex,
    },
    ByDbSize {
        num_bytes_to_prune: usize,
    },
}

pub(crate) fn should_prune<S: StorageBackend>(
    tangle: &Tangle<S>,
    storage: &S,
    ledger_index: LedgerIndex,
    milestones_to_keep: u32,
    pruning_config: &PruningConfig,
) -> Result<PruningTask, PruningSkipReason> {
    if !pruning_config.milestones().enabled() && !pruning_config.db_size().enabled() {
        return Err(PruningSkipReason::Disabled);
    }

    let pruning_by_size = if pruning_config.db_size().enabled() {
        let prev = Duration::from_secs(PREVIOUS_PRUNING_BY_SIZE.load(Ordering::Relaxed));
        // Panic: should not cause problems on properly set up hosts.
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        if now < prev + pruning_config.db_size().cooldown_time() {
            Err(PruningSkipReason::Cooldown)
        } else {
            let actual_size = {
                if let Ok(size) = storage.size() {
                    if let Some(size) = size {
                        Ok(size)
                    } else {
                        Err(PruningSkipReason::SizeMetricUnavailable)
                    }
                } else {
                    Err(PruningSkipReason::SizeMetricUnsupported)
                }
            };

            match actual_size {
                Ok(actual_size) => {
                    let threshold_size = pruning_config.db_size().target_size();

                    log::debug!("Storage size: actual {actual_size} threshold {threshold_size}");

                    if actual_size < threshold_size {
                        Err(PruningSkipReason::BelowTargetSizeThreshold)
                    } else {
                        // Panic: cannot underflow due to actual_size >= threshold_size.
                        let excess_size = actual_size - threshold_size;
                        let num_bytes_to_prune = excess_size
                            + (pruning_config.db_size().threshold_percentage() as f64 / 100.0 * threshold_size as f64)
                                as usize;

                        log::debug!("Num bytes to prune: {num_bytes_to_prune}");

                        // Store the time we issued a pruning-by-size.
                        PREVIOUS_PRUNING_BY_SIZE.store(now.as_secs(), Ordering::Relaxed);

                        Ok(PruningTask::ByDbSize { num_bytes_to_prune })
                    }
                }
                Err(reason) => Err(reason),
            }
        }
    } else {
        Err(PruningSkipReason::Disabled)
    };

    if pruning_by_size.is_err() && pruning_config.milestones().enabled() {
        let pruning_index = *tangle.get_pruning_index() + 1;
        let target_index_threshold = pruning_index + milestones_to_keep;

        if *ledger_index < target_index_threshold {
            Err(PruningSkipReason::BelowTargetIndexThreshold)
        } else {
            let target_pruning_index = *ledger_index - milestones_to_keep;

            Ok(PruningTask::ByIndexRange {
                start_index: pruning_index.into(),
                target_index: if target_pruning_index > pruning_index + PRUNING_BATCH_SIZE_MAX {
                    (pruning_index + PRUNING_BATCH_SIZE_MAX).into()
                } else {
                    target_pruning_index.into()
                },
            })
        }
    } else {
        pruning_by_size
    }
}
