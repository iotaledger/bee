// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::HeartbeatPacket, peer::PeerManager, sender::Sender, storage::StorageBackend, MetricsWorker,
        PeerManagerResWorker,
    },
};

use bee_gossip::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use std::{any::TypeId, convert::Infallible, time::Duration};

const HEARTBEAT_SEND_INTERVAL: Duration = Duration::from_secs(30);

pub(crate) async fn new_heartbeat<B: StorageBackend>(
    tangle: &Tangle<B>,
    peer_manager: &PeerManager,
) -> HeartbeatPacket {
    let connected_peers = peer_manager.connected_peers().await;
    let synced_peers = peer_manager.synced_peers().await;

    HeartbeatPacket::new(
        *tangle.get_solid_milestone_index(),
        *tangle.get_pruning_index(),
        *tangle.get_latest_milestone_index(),
        connected_peers,
        synced_peers,
    )
}

pub(crate) async fn send_heartbeat(
    heartbeat: &HeartbeatPacket,
    peer_id: &PeerId,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
) {
    Sender::<HeartbeatPacket>::send(heartbeat, peer_id, peer_manager, metrics).await;
}

pub(crate) async fn broadcast_heartbeat<B: StorageBackend>(
    tangle: &Tangle<B>,
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
) {
    let heartbeat = new_heartbeat(tangle, peer_manager).await;
    let peers = peer_manager.get_all().await;

    for peer_id in peers.iter().map(|p| (*p).id()) {
        send_heartbeat(&heartbeat, peer_id, peer_manager, metrics).await;
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
        let tangle = node.resource::<Tangle<N::Backend>>();
        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(HEARTBEAT_SEND_INTERVAL)));

            while ticker.next().await.is_some() {
                broadcast_heartbeat(&tangle, &peer_manager, &metrics).await;
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
