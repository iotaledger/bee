// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::{storage::StorageBackend, MessageRequesterWorker, RequestedMessages};

use bee_ledger::workers::LedgerWorker;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{any::TypeId, convert::Infallible, time::Duration};

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
            TypeId::of::<MessageRequesterWorker>(),
            TypeId::of::<LedgerWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(Duration::from_secs(config))));

            while ticker.next().await.is_some() {
                let snapshot_index = *tangle.get_snapshot_index();
                let confirmed_milestone_index = *tangle.get_confirmed_milestone_index();
                let solid_milestone_index = *tangle.get_solid_milestone_index();
                let latest_milestone_index = *tangle.get_latest_milestone_index();

                let status = if confirmed_milestone_index == latest_milestone_index {
                    format!("Synchronized and confirmed at {}", latest_milestone_index)
                } else {
                    let confirmed_progress = ((confirmed_milestone_index - snapshot_index) as f64 * 100.0
                        / (latest_milestone_index - snapshot_index) as f64)
                        as u8;
                    let solid_progress = ((solid_milestone_index - snapshot_index) as f32 * 100.0
                        / (latest_milestone_index - snapshot_index) as f32)
                        as u8;
                    format!(
                        "Synchronizing from {} to {}: confirmed {} ({}%) and solid {} ({}%) - Requested {}",
                        snapshot_index,
                        latest_milestone_index,
                        confirmed_milestone_index,
                        confirmed_progress,
                        solid_milestone_index,
                        solid_progress,
                        requested_messages.len().await,
                    )
                };

                info!("{} - Tips {}.", status, tangle.non_lazy_tips_num().await);
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
