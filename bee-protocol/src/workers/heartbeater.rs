// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{packets::HeartbeatPacket, peer::PeerManager, sender::Sender, MetricsWorker, PeerManagerResWorker},
};

use bee_network::PeerId;
use bee_runtime::{node::Node, resource::ResourceHandle, worker::Worker};
use bee_tangle::{storage::StorageBackend, Tangle, TangleWorker};

use async_trait::async_trait;
use backstage::core::{Actor, ActorError, ActorResult, IntervalChannel, Rt, SupHandle};
use futures::stream::StreamExt;
use log::info;

use std::{any::TypeId, convert::Infallible, marker::PhantomData, time::Duration};

const _HEARTBEAT_SEND_INTERVAL: u64 = 30; // In seconds.
const _HEARTBEAT_RECEIVE_INTERVAL: u64 = 100; // In seconds.
const CHECK_HEARTBEATS_INTERVAL: u64 = Duration::from_secs(5).as_millis() as u64;

pub(crate) async fn new_heartbeat<B: StorageBackend>(
    peer_manager: &PeerManager,
    tangle: &Tangle<B>,
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
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    to: &PeerId,
    heartbeat: HeartbeatPacket,
) {
    Sender::<HeartbeatPacket>::send(peer_manager, metrics, to, heartbeat).await;
}

pub(crate) async fn broadcast_heartbeat<B: StorageBackend>(
    peer_manager: &PeerManager,
    metrics: &NodeMetrics,
    tangle: &Tangle<B>,
) {
    let heartbeat = new_heartbeat(peer_manager, tangle).await;

    // TODO bring it back
    //    peer_manager.for_each_peer(|peer_id, _| async {
    for (peer_id, _) in peer_manager.0.read().await.peers.iter() {
        send_heartbeat(peer_manager, metrics, peer_id, heartbeat.clone()).await
    }
}

pub struct HeartbeaterActor<B: StorageBackend> {
    _marker: PhantomData<(B,)>,
}

impl<B: StorageBackend> Default for HeartbeaterActor<B> {
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

#[async_trait]
impl<S, B: StorageBackend> Actor<S> for HeartbeaterActor<B>
where
    S: SupHandle<Self>,
{
    type Data = (
        ResourceHandle<Tangle<B>>,
        ResourceHandle<PeerManager>,
        ResourceHandle<NodeMetrics>,
    );
    type Channel = IntervalChannel<CHECK_HEARTBEATS_INTERVAL>;

    async fn init(&mut self, rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        // This should be the ID of the supervisor.
        let parent_id = rt
            .parent_id()
            .ok_or_else(|| ActorError::aborted_msg("gossip actor has no parent"))?;

        // The event bus should be under the supervisor's ID.
        let tangle = rt
            .lookup::<ResourceHandle<Tangle<B>>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("tangle is not available"))?;

        // The peer manager should be under the supervisor's ID.
        let peer_manager = rt
            .lookup::<ResourceHandle<PeerManager>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("peer manager is not available"))?;

        // The node metrics should be under the supervisor's ID.
        let node_metrics = rt
            .lookup::<ResourceHandle<NodeMetrics>>(parent_id)
            .await
            .ok_or_else(|| ActorError::exit_msg("node metrics is not available"))?;

        Ok((tangle, peer_manager, node_metrics))
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, (tangle, peer_manager, metrics): Self::Data) -> ActorResult<()> {
        info!("Running.");

        while rt.inbox_mut().next().await.is_some() {
            broadcast_heartbeat::<B>(&peer_manager, &metrics, &tangle).await;
        }

        info!("Stopped.");

        Ok(())
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
        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            shutdown.await.unwrap();

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
