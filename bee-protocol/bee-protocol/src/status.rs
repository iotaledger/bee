// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible, time::Duration};

use async_trait::async_trait;
use bee_ledger::consensus::ConsensusWorker;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::{storage::StorageBackend, BlockRequesterWorker, RequestedBlocks};

#[derive(Default)]
pub(crate) struct StatusWorker;

#[async_trait]
impl<N: Node> Worker<N> for StatusWorker
where
    N::Backend: StorageBackend,
{
    type Config = u64;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<BlockRequesterWorker>(),
            TypeId::of::<ConsensusWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_blocks = node.resource::<RequestedBlocks>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(Duration::from_secs(config))));

            while ticker.next().await.is_some() {
                let snapshot_index = *tangle.get_snapshot_index();
                let solid_milestone_index = *tangle.get_solid_milestone_index();
                let confirmed_milestone_index = *tangle.get_confirmed_milestone_index();
                let latest_milestone_index = *tangle.get_latest_milestone_index();
                let non_lazy_tips_num = tangle.non_lazy_tips_num().await;

                let status = if confirmed_milestone_index == latest_milestone_index {
                    format!("Synchronized and confirmed at {}", latest_milestone_index)
                } else {
                    let solid_progress = ((solid_milestone_index - snapshot_index) as f64 * 100.0
                        / (latest_milestone_index - snapshot_index) as f64)
                        as u8;
                    let confirmed_progress = ((confirmed_milestone_index - snapshot_index) as f64 * 100.0
                        / (latest_milestone_index - snapshot_index) as f64)
                        as u8;

                    format!(
                        "Synchronizing from {} to {}: confirmed {} ({}%) and solid {} ({}%) - Requested {}",
                        snapshot_index,
                        latest_milestone_index,
                        confirmed_milestone_index,
                        confirmed_progress,
                        solid_milestone_index,
                        solid_progress,
                        requested_blocks.len(),
                    )
                };

                info!("{} - Tips {}.", status, non_lazy_tips_num);
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
