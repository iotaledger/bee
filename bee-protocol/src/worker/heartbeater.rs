// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    helper,
    peer::PeerManager,
    storage::StorageBackend,
    worker::{MetricsWorker, PeerManagerResWorker},
    ProtocolMetrics,
};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{any::TypeId, convert::Infallible, time::Duration};

const _HEARTBEAT_SEND_INTERVAL_SEC: u64 = 30;
const _HEARTBEAT_RECEIVE_INTERVAL_SEC: u64 = 100;
const CHECK_HEARTBEATS_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub(crate) struct HeartbeaterWorker {}

#[async_trait]
impl<N: Node> Worker<N> for HeartbeaterWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
            TypeId::of::<MetricsWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_HEARTBEATS_INTERVAL_SEC))),
            );

            while ticker.next().await.is_some() {
                // TODO real impl
                helper::broadcast_heartbeat(
                    &peer_manager,
                    &metrics,
                    tangle.get_latest_solid_milestone_index(),
                    tangle.get_pruning_index(),
                    tangle.get_latest_milestone_index(),
                )
                .await;
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
