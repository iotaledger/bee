// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig,
    constants::{
        ADDITIONAL_PRUNING_THRESHOLD, SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE, SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST,
    },
    snapshot,
    pruning::prune_database,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common::{node::Node, worker::Worker};
use bee_protocol::{tangle::MsTangle, Milestone, MilestoneIndex, TangleWorker};
use bee_storage::storage::Backend;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info, warn};

use std::{convert::Infailible, any::TypeId};

pub(crate) struct SnapshotWorkerEvent(pub(crate) Milestone);

pub(crate) struct SnapshotWorker {
    pub(crate) tx: flume::Sender<SnapshotWorkerEvent>,
}

fn should_snapshot<B: Backend>(tangle: &MsTangle<B>, index: MilestoneIndex, config: &SnapshotConfig, depth: u32) -> bool {
    let solid_index = *index;
    let snapshot_index = *tangle.get_snapshot_index();
    let pruning_index = *tangle.get_pruning_index();
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    if (solid_index < depth + snapshot_interval)
        || (solid_index - depth) < pruning_index + 1 + SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST
    {
        // Not enough history to calculate solid entry points.
        return false;
    }

    solid_index - (depth + snapshot_interval) >= snapshot_index
}

fn should_prune<B: Backend>(tangle: &MsTangle<B>, mut index: MilestoneIndex, config: &SnapshotConfig, delay: u32) -> bool {
    if !config.pruning().enabled() {
        return false;
    }

    if *index <= delay {
        return false;
    }

    // Pruning happens after creating the snapshot so the metadata should provide the latest index.
    if *tangle.get_snapshot_index() < SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1 {
        return false;
    }

    let target_index_max = MilestoneIndex(
        *tangle.get_snapshot_index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1,
    );

    if index > target_index_max {
        index = target_index_max;
    }

    if tangle.get_pruning_index() >= index {
        return false;
    }

    // We prune in "ADDITIONAL_PRUNING_THRESHOLD" steps to recalculate the solid_entry_points.
    if *tangle.get_entry_point_index() + ADDITIONAL_PRUNING_THRESHOLD + 1 > *index {
        return false;
    }

    true
}

#[async_trait]
impl<N: Node> Worker<N> for SnapshotWorker {
    type Config = SnapshotConfig;
    type Error = Infailible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let depth = if config.depth() < SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE {
                warn!(
                    "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                    config.depth(),
                    SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE
                );
                SOLID_ENTRY_POINT_CHECK_THRESHOLD_FUTURE
            } else {
                config.depth()
            };
            let delay_min =
                config.depth() + SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1;
            let delay = if config.pruning().delay() < delay_min {
                warn!(
                    "Configuration value for \"delay\" is too low ({}), value changed to {}.",
                    config.pruning().delay(),
                    delay_min
                );
                delay_min
            } else {
                config.pruning().delay()
            };

            while let Some(SnapshotWorkerEvent(milestone)) = receiver.next().await {
                if should_snapshot(&tangle, milestone.index(), &config, depth) {
                    if let Err(e) = snapshot(config.path(), *milestone.index() - depth) {
                        error!("Failed to create snapshot: {:?}.", e);
                    }
                }
                if should_prune(&tangle, milestone.index(), &config, delay) {
                    if let Err(e) = prune_database(&tangle, MilestoneIndex(*milestone.index() - delay)) {
                        error!("Failed to prune database: {:?}.", e);
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
