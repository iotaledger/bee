// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    tangle::MsTangle,
    worker::{MessageRequesterWorker, RequestedMessages, TangleWorker},
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::StreamExt;
use log::info;
use tokio::time::interval;

use std::{any::TypeId, convert::Infallible, time::Duration};

#[derive(Default)]
pub(crate) struct StatusWorker;

#[async_trait]
impl<N: Node> Worker<N> for StatusWorker {
    type Config = u64;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>(), TypeId::of::<MessageRequesterWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_messages = node.resource::<RequestedMessages>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, interval(Duration::from_secs(config)));

            while ticker.next().await.is_some() {
                let snapshot_index = *tangle.get_snapshot_index();
                let latest_solid_milestone_index = *tangle.get_latest_solid_milestone_index();
                let latest_milestone_index = *tangle.get_latest_milestone_index();

                // TODO Threshold
                // TODO use tangle synced method
                let status = if latest_solid_milestone_index == latest_milestone_index {
                    format!("Synchronized at {}", latest_milestone_index)
                } else {
                    let progress = ((latest_solid_milestone_index - snapshot_index) as f32 * 100.0
                        / (latest_milestone_index - snapshot_index) as f32) as u8;
                    format!(
                        "Synchronizing {}..{}..{} ({}%) - Requested {}",
                        snapshot_index,
                        latest_solid_milestone_index,
                        latest_milestone_index,
                        progress,
                        requested_messages.len(),
                    )
                };

                info!("{} - Tips {}.", status, tangle.non_lazy_tips_num().await);
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
