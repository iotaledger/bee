// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    helper,
    peer::{Peer, PeerManager},
    storage::StorageBackend,
    worker::{
        HasherWorker, MessageResponderWorker, MetricsWorker, MilestoneRequesterWorker, MilestoneResponderWorker,
        PeerManagerResWorker, PeerWorker, RequestedMilestones,
    },
    ProtocolMetrics,
};

use bee_network::{Event, NetworkListener};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{info, trace, warn};
use tokio::task::spawn;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible, sync::Arc};

pub(crate) struct PeerManagerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for PeerManagerWorker
where
    N::Backend: StorageBackend,
{
    type Config = NetworkListener;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<HasherWorker>(),
            TypeId::of::<MessageResponderWorker>(),
            TypeId::of::<MilestoneResponderWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let peer_manager = node.resource::<PeerManager>();
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<ProtocolMetrics>();
        let hasher = node.worker::<HasherWorker>().unwrap().tx.clone();
        let message_responder = node.worker::<MessageResponderWorker>().unwrap().tx.clone();
        let milestone_responder = node.worker::<MilestoneResponderWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(config));

            while let Some(event) = receiver.next().await {
                trace!("Received event {:?}.", event);

                match event {
                    Event::PeerAdded { id, info } => {
                        // TODO check if not already added ?
                        info!("Added peer {}.", info.alias);

                        let peer = Arc::new(Peer::new(id, info));
                        peer_manager.add(peer).await;
                    }
                    Event::PeerRemoved { id } => {
                        if let Some(peer) = peer_manager.remove(&id).await {
                            info!("Removed peer {}.", peer.0.alias());
                        }
                    }
                    Event::PeerConnected {
                        id,
                        address: _,
                        gossip_in: receiver,
                        gossip_out: sender,
                    } => {
                        // TODO write a get_mut peer manager method
                        if let Some(peer) = peer_manager.peers.write().await.get_mut(&id) {
                            let (shutdown_tx, shutdown_rx) = oneshot::channel();

                            peer.1 = Some((sender, shutdown_tx));

                            spawn(
                                PeerWorker::new(
                                    peer.0.clone(),
                                    metrics.clone(),
                                    hasher.clone(),
                                    message_responder.clone(),
                                    milestone_responder.clone(),
                                    milestone_requester.clone(),
                                )
                                .run(
                                    tangle.clone(),
                                    requested_milestones.clone(),
                                    receiver,
                                    shutdown_rx,
                                ),
                            );

                            info!("Connected peer {}.", peer.0.alias());
                        }

                        // TODO can't do it in the if because of deadlock, but it's not really right to do it here.
                        helper::send_heartbeat(&*peer_manager, &*metrics, &*tangle, &id).await;
                    }
                    Event::PeerDisconnected { id } => {
                        // TODO write a get_mut peer manager method
                        if let Some(peer) = peer_manager.peers.write().await.get_mut(&id) {
                            if let Some((_, shutdown)) = peer.1.take() {
                                if let Err(e) = shutdown.send(()) {
                                    warn!("Sending shutdown to {} failed: {:?}.", peer.0.alias(), e);
                                }
                            }
                            info!("Disconnected peer {}.", peer.0.alias());
                        }
                    }
                    _ => (), // Ignore all other events for now
                }
            }

            info!("Stopped.");
        });

        Ok(Self {})
    }
}
