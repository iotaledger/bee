// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{error::Error, prune::prune};

use crate::{
    event::LatestSolidMilestoneChanged, pruning::config::PruningConfig, storage::StorageBackend, MsTangle, TangleWorker,
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_snapshot::config::SnapshotConfig;

use async_trait::async_trait;
use log::{error, info, warn};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, StreamExt};

use std::{any::TypeId, convert::Infallible, path::Path};

// pub(crate) const PAST_CONE_THRESHOLD: u32 = 5;
pub(crate) const SNAPSHOT_DEPTH_MIN: u32 = 5;
pub(crate) const PRUNING_INTERVAL: u32 = 50;

#[derive(Debug)]
pub struct PrunerWorkerInput(pub(crate) LatestSolidMilestoneChanged);

pub struct PrunerWorker {}

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
        let (tx, rx) = mpsc::unbounded_channel();
        let tangle = node.resource::<MsTangle<N::Backend>>().clone();
        let bus = node.bus();
        let (snapshot_config, pruning_config) = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            // Change misconfigured snapshot `depth`s.
            let snapshot_depth = if snapshot_config.depth() < SNAPSHOT_DEPTH_MIN {
                warn!(
                    "Configuration value for \"depth\" is too low ({}), value changed to {}.",
                    snapshot_config.depth(),
                    SNAPSHOT_DEPTH_MIN
                );
                SNAPSHOT_DEPTH_MIN
            } else {
                snapshot_config.depth()
            };

            // Change misconfigured pruning `delay`s.
            let pruning_delay_min = 2 * snapshot_depth + PRUNING_INTERVAL;
            let pruning_delay = if pruning_config.delay() < pruning_delay_min {
                warn!(
                    "Configuration value for \"delay\" is too low ({}), value changed to {}.",
                    pruning_config.delay(),
                    pruning_delay_min
                );
                pruning_delay_min
            } else {
                pruning_config.delay()
            };

            // Ensure that `pruning_delay` >> `snapshot_depth` is an invariant.
            debug_assert!(pruning_delay > snapshot_depth);

            // The following event-loop runs whenever a new milestone has been solidified, i.e.
            // event `LatestSolidMilestoneChanged` occurred.
            while let Some(PrunerWorkerInput(lsms)) = receiver.next().await {
                if let Some(snapshot_index) = should_snapshot(&tangle, lsms.index, snapshot_depth, &snapshot_config) {
                    if let Err(e) = snapshot(snapshot_config.full_path(), snapshot_index) {
                        error!("Failed to create snapshot: {:?}.", e);
                    }
                }
                if let Some(pruning_target_index) = should_prune(&tangle, lsms.index, pruning_delay, &pruning_config) {
                    if let Err(e) = prune(&tangle, pruning_target_index).await {
                        error!("Failed to prune database: {:?}.", e);
                    }
                }
            }

            info!("Stopped.");
        });

        // Subscribe to the `LatestSolidMilestoneChanged` event.
        bus.add_listener::<Self, _, _>(move |lsms: &LatestSolidMilestoneChanged| {
            if let Err(e) = tx.send(PrunerWorkerInput(lsms.clone())) {
                warn!("Failed to send milestone {} to snapshot worker: {:?}.", lsms.index, e)
            }
        });

        Ok(Self {})
    }
}

fn should_snapshot<B: StorageBackend>(
    tangle: &MsTangle<B>,
    index: MilestoneIndex,
    depth: u32,
    config: &SnapshotConfig,
) -> Option<MilestoneIndex> {
    let current_solid_index = *index;

    let snapshot_index = *tangle.get_snapshot_index();
    debug_assert!(current_solid_index > snapshot_index);

    // If the node is unsync we snapshot less often.
    let snapshot_interval = if tangle.is_synced() {
        config.interval_synced()
    } else {
        config.interval_unsynced()
    };

    // Do not snapshot without enough depth. This will only happen for a freshly started node.
    if current_solid_index < depth {
        return None;
    }

    // Do not snapshot out of interval.
    if current_solid_index % snapshot_interval != 0 {
        return None;
    }

    Some(MilestoneIndex(current_solid_index - depth))
}

fn should_prune<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    index: MilestoneIndex,
    delay: u32,
    config: &PruningConfig,
) -> Option<MilestoneIndex> {
    // Do not prune if pruning is disabled in the config.
    if !config.enabled() {
        return None;
    }

    let current_solid_index = *index;

    // Do not prune if there isn't old enough data to prune yet. This will only happen for a freshly started node.
    if current_solid_index < delay {
        return None;
    }

    // Do not prune out of interval.
    if current_solid_index % PRUNING_INTERVAL != 0 {
        return None;
    }

    // Return the `target_index`, i.e. the `MilestoneIndex` up to which the database can be savely pruned.
    Some(MilestoneIndex(current_solid_index - delay))
}

fn snapshot(_path: &Path, _snapshot_index: MilestoneIndex) -> Result<(), Error> {
    info!("Snapshotting...");
    // TODO
    Ok(())
}
