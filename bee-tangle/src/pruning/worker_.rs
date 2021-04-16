// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MsTangle, TangleWorker};

use bee_message::milestone::{Milestone, MilestoneIndex};
use bee_runtime::{event::Bus, node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_snapshot::config::SnapshotConfig;
use bee_storage::backend::StorageBackend;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{error, info, warn};
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

use super::{ADDITIONAL_PRUNING_THRESHOLD, SEP_CHECK_THRESHOLD_FUTURE, SEP_CHECK_THRESHOLD_PAST};

pub(crate) struct SnapshotWorkerEvent(pub(crate) Milestone);

pub(crate) struct SnapshotWorker {
    pub(crate) tx: mpsc::Sender<SnapshotWorkerEvent>,
}

fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    index: MilestoneIndex,
    config: &SnapshotConfig,
    depth: u32,
) -> bool {
    // ???
    let solid_index = *index;

    // that is the milestone index of the last snapshot (-> creates a snapshot file)
    let snapshot_index = *tangle.get_snapshot_index();

    // that is the milestone index of the last pruning
    let pruning_index = *tangle.get_pruning_index();

    // ??? if synced (LMI==SMI), then this interval is set to 50, otherwise to 1000 (in the config)
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    // The first condition ensures, that no snapshot takes place before we reach (depth + snapshot_interval)
    // The second condition ensures, that we have enough history since the last pruning

    if (solid_index < depth + snapshot_interval) || (solid_index - depth) < pruning_index + 1 + SEP_CHECK_THRESHOLD_PAST
    {
        // Not enough history to calculate solid entry points.
        return false;
    }

    // snapshot_index is fixed (at the last performed snapshot_index)
    // depth = 50
    // snapshot_interval = 50
    // => when synced to a snapshot every 100 milestones
    // solid_index = 500
    // snapshot_index = 400
    // 500      - (50    + 50               ) >= 400

    solid_index - (depth + snapshot_interval) >= snapshot_index
}

fn should_prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    mut index: MilestoneIndex,
    config: &SnapshotConfig,
    delay: u32,
) -> bool {
    // Do not prune if disabled in config
    if !config.pruning().enabled() {
        return false;
    }

    // Do not prune if
    if *index <= delay {
        return false;
    }

    // Pruning happens after creating the snapshot so the metadata should provide the latest index.
    if *tangle.get_snapshot_index() < SEP_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1 {
        return false;
    }

    let target_index_max =
        MilestoneIndex(*tangle.get_snapshot_index() - SEP_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1);

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
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();
        let bus = node.bus();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let depth = if config.depth() < SEP_CHECK_THRESHOLD_FUTURE {
                warn!(
                    "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                    config.depth(),
                    SEP_CHECK_THRESHOLD_FUTURE
                );
                SEP_CHECK_THRESHOLD_FUTURE
            } else {
                config.depth()
            };
            let delay_min = config.depth() + SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST + ADDITIONAL_PRUNING_THRESHOLD + 1;
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

        // bus.add_listener(move |solid_milestone: &LatestSolidMilestoneChanged| {
        //     if let Err(e) = snapshot_worker.send(worker::SnapshotWorkerEvent(solid_milestone.0.clone())) {
        //         warn!(
        //             "Failed to send milestone {} to snapshot worker: {:?}.",
        //             *solid_milestone.0.index(),
        //             e
        //         )
        //     }
        // });

        Ok(Self { tx })
    }
}
