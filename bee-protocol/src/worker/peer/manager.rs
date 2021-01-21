// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{Peer, PeerManager},
    storage::StorageBackend,
    worker::{
        HasherWorker, MessageResponderWorker, MetricsWorker, MilestoneRequesterWorker, MilestoneResponderWorker,
        PeerManagerResWorker, PeerWorker, RequestedMilestones, TangleWorker,
    },
    ProtocolMetrics,
};

use bee_network::{Event, NetworkListener, ShortId};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{info, trace, warn};
use tokio::{sync::mpsc, task::spawn};
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
                    Event::PeerAdded { id, info: _ } => {
                        info!("Added peer: {}", id.short());
                    }
                    Event::PeerRemoved { id } => {
                        info!("Removed peer: {}", id.short());
                    }
                    Event::PeerConnected { id, address } => {
                        // // TODO check if not already added ?

                        let peer = Arc::new(Peer::new(id, address));

                        let (receiver_tx, receiver_rx) = mpsc::unbounded_channel();
                        let (shutdown_tx, shutdown_rx) = oneshot::channel();

                        spawn(
                            PeerWorker::new(
                                peer.clone(),
                                metrics.clone(),
                                peer_manager.clone(),
                                hasher.clone(),
                                message_responder.clone(),
                                milestone_responder.clone(),
                                milestone_requester.clone(),
                            )
                            .run(
                                tangle.clone(),
                                requested_milestones.clone(),
                                receiver_rx,
                                shutdown_rx,
                            ),
                        );

                        peer_manager.add(peer, receiver_tx, shutdown_tx).await;
                    }
                    Event::PeerDisconnected { id } => {
                        if let Some((_, _, shutdown)) = peer_manager.remove(&id).await {
                            if let Err(e) = shutdown.send(()) {
                                warn!("Sending shutdown to {} failed: {:?}.", id.short(), e);
                            }
                        }
                    }
                    Event::MessageReceived { message, from } => {
                        if let Some(peer) = peer_manager.get(&from).await {
                            if let Err(e) = (*peer).1.send(message) {
                                warn!("Sending PeerWorkerEvent::Message to {} failed: {}.", from.short(), e);
                            }
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
