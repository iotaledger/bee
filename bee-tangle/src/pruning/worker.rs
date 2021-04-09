// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// use crate::{
//     config::SnapshotConfig,
//     pruning::prune_database,
//     snapshot,
// };

use crate::{
    event::ConfirmedMilestoneChanged,
    pruning::{
        config::PruningConfig,
        constants::{PRUNING_THRESHOLD, SOLID_ENTRY_POINT_THRESHOLD_FUTURE, SOLID_ENTRY_POINT_THRESHOLD_PAST},
    },
    storage::StorageBackend,
    MsTangle, TangleWorker,
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_snapshot::config::SnapshotConfig;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};

use std::{any::TypeId, convert::Infallible};

pub struct PrunerWorkerEvent(pub(crate) ConfirmedMilestoneChanged);

pub struct PrunerWorker {}

fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    index: MilestoneIndex,
    depth: u32,
    config: &SnapshotConfig,
) -> bool {
    let solid_index = *index;
    let snapshot_index = *tangle.get_snapshot_index();
    let pruning_index = *tangle.get_pruning_index();
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    if (solid_index < depth + snapshot_interval)
        || (solid_index - depth) < pruning_index + 1 + SOLID_ENTRY_POINT_THRESHOLD_PAST
    {
        // Not enough history to calculate solid entry points.
        return false;
    }

    solid_index - (depth + snapshot_interval) >= snapshot_index
}

fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    mut index: MilestoneIndex,
    delay: u32,
    config: &PruningConfig,
) -> bool {
    if !config.enabled() {
        return false;
    }

    if *index <= delay {
        return false;
    }

    // Pruning happens after creating the snapshot so the metadata should provide the latest index.
    if *tangle.get_snapshot_index() < SOLID_ENTRY_POINT_THRESHOLD_PAST + PRUNING_THRESHOLD + 1 {
        return false;
    }

    let target_index_max =
        MilestoneIndex(*tangle.get_snapshot_index() - SOLID_ENTRY_POINT_THRESHOLD_PAST - PRUNING_THRESHOLD - 1);

    if index > target_index_max {
        index = target_index_max;
    }

    if tangle.get_pruning_index() >= index {
        return false;
    }

    // We prune in "PRUNING_THRESHOLD" steps to recalculate the solid_entry_points.
    if *tangle.get_entry_point_index() + PRUNING_THRESHOLD + 1 > *index {
        return false;
    }

    true
}

#[async_trait]
impl<N: Node> Worker<N> for PrunerWorker
where
    N::Backend: StorageBackend,
{
    type Config = (SnapshotConfig, PruningConfig);
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();
        let bus = node.bus();
        let (snapshot_config, pruning_config) = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let depth = if snapshot_config.depth() < SOLID_ENTRY_POINT_THRESHOLD_FUTURE {
                warn!(
                    "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                    snapshot_config.depth(),
                    SOLID_ENTRY_POINT_THRESHOLD_FUTURE
                );
                SOLID_ENTRY_POINT_THRESHOLD_FUTURE
            } else {
                snapshot_config.depth()
            };
            let delay_min = snapshot_config.depth() + SOLID_ENTRY_POINT_THRESHOLD_PAST + PRUNING_THRESHOLD + 1;
            let delay = if pruning_config.delay() < delay_min {
                warn!(
                    "Configuration value for \"delay\" is too low ({}), value changed to {}.",
                    pruning_config.delay(),
                    delay_min
                );
                delay_min
            } else {
                pruning_config.delay()
            };

            while let Some(PrunerWorkerEvent(event)) = receiver.next().await {
                if should_snapshot(&tangle, event.index, depth, &snapshot_config) {
                    // if let Err(e) = snapshot(snapshot_config.path(), event.index - depth) {
                    //     error!("Failed to create snapshot: {:?}.", e);
                    // }
                }
                if should_prune(&tangle, event.index, delay, &pruning_config) {
                    // if let Err(e) = prune_database(&tangle, MilestoneIndex(*event.index - delay)) {
                    //     error!("Failed to prune database: {:?}.", e);
                    // }
                }
            }

            info!("Stopped.");
        });

        bus.add_listener::<Self, _, _>(move |event: &ConfirmedMilestoneChanged| {
            if let Err(e) = tx.send(PrunerWorkerEvent(event.clone())) {
                warn!("Failed to send milestone {} to snapshot worker: {:?}.", event.index, e)
            }
        });

        Ok(Self {})
    }
}
