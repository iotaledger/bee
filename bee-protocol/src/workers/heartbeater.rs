// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::HeartbeatPacket, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{any::TypeId, convert::Infallible, time::Duration};

const _HEARTBEAT_SEND_INTERVAL: u64 = 30; // In seconds.
const _HEARTBEAT_RECEIVE_INTERVAL: u64 = 100; // In seconds.
const CHECK_HEARTBEATS_INTERVAL: u64 = 5; // In seconds.

pub async fn send_heartbeat<B: StorageBackend>(
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &MsTangle<B>,
    to: &PeerId,
) {
    let connected_peers = peer_manager.connected_peers().await;
    let synced_peers = peer_manager.synced_peers().await;

    Sender::<HeartbeatPacket>::send(
        peer_manager,
        metrics,
        to,
        HeartbeatPacket::new(
            *tangle.get_solid_milestone_index(),
            *tangle.get_pruning_index(),
            *tangle.get_latest_milestone_index(),
            connected_peers,
            synced_peers,
        ),
    )
    .await;
}

pub async fn broadcast_heartbeat<B: StorageBackend>(
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &MsTangle<B>,
) {
    // TODO bring it back
    //    peer_manager.for_each_peer(|peer_id, _| async {
    for (peer_id, _) in peer_manager.0.read().await.peers.iter() {
        send_heartbeat(peer_manager, metrics, tangle, &peer_id).await
    }
}

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
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(
                shutdown,
                IntervalStream::new(interval(Duration::from_secs(CHECK_HEARTBEATS_INTERVAL))),
            );

            while ticker.next().await.is_some() {
                // TODO real impl
                broadcast_heartbeat(&peer_manager, &metrics, &tangle).await;
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
