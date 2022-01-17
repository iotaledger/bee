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

const CHECK_HEARTBEATS_INTERVAL: Duration = Duration::from_secs(5);
const _HEARTBEAT_SEND_INTERVAL: Duration = Duration::from_secs(30);
const _HEARTBEAT_RECEIVE_INTERVAL: Duration = Duration::from_secs(100);

pub(crate) fn new_heartbeat<B: StorageBackend>(peer_manager: &PeerManager, tangle: &Tangle<B>) -> HeartbeatPacket {
    let connected_peers = peer_manager.connected_peers();
    let synced_peers = peer_manager.synced_peers();

    HeartbeatPacket::new(
        *tangle.get_solid_milestone_index(),
        *tangle.get_pruning_index(),
        *tangle.get_latest_milestone_index(),
        connected_peers,
        synced_peers,
    )
}

pub(crate) fn send_heartbeat(
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    peer_id: &PeerId,
    heartbeat: &HeartbeatPacket,
) {
    Sender::<HeartbeatPacket>::send(heartbeat, peer_id, peer_manager, metrics);
}

pub(crate) fn broadcast_heartbeat<B: StorageBackend>(
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &Tangle<B>,
) {
    let heartbeat = new_heartbeat(peer_manager, tangle);

    peer_manager.for_each(|peer_id, _| send_heartbeat(peer_manager, metrics, peer_id, &heartbeat));
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

            let mut ticker = ShutdownStream::new(shutdown, IntervalStream::new(interval(CHECK_HEARTBEATS_INTERVAL)));

            while ticker.next().await.is_some() {
                // TODO real impl
                broadcast_heartbeat(&peer_manager, &metrics, &tangle);
            }

            info!("Stopped.");
        });

        Ok(Self::default())
    }
}
