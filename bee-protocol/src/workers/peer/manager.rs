// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{metrics::NodeMetrics, peer::Peer},
    workers::{
        heartbeater::{new_heartbeat, send_heartbeat},
        peer::PeerManager,
        storage::StorageBackend,
        HasherWorker, MessageResponderWorker, MetricsWorker, MilestoneRequesterWorker, MilestoneResponderWorker,
        PeerManagerResWorker, PeerWorker, RequestedMilestones,
    },
};

use bee_autopeering::event::{Event as AutopeeringEvent, EventRx as AutopeeringEventRx};
use bee_network::{
    alias, Command, Event as NetworkEvent, NetworkCommandSender, NetworkEventReceiver as NetworkEventRx, PeerRelation,
    ServiceHost,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{info, trace, warn};
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible, sync::Arc};

pub(crate) struct PeerManagerConfig {
    pub(crate) network_rx: NetworkEventRx,
    pub(crate) peering_rx: Option<AutopeeringEventRx>,
    pub(crate) network_name: String,
}

pub(crate) struct PeerManagerWorker {}

#[async_trait]
impl<N: Node> Worker<N> for PeerManagerWorker
where
    N::Backend: StorageBackend,
{
    type Config = PeerManagerConfig;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<ServiceHost>(),
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
        let tangle = node.resource::<Tangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();
        let metrics = node.resource::<NodeMetrics>();
        let network_command_tx = node.resource::<NetworkCommandSender>();

        let hasher = node.worker::<HasherWorker>().unwrap().tx.clone();
        let message_responder = node.worker::<MessageResponderWorker>().unwrap().tx.clone();
        let milestone_responder = node.worker::<MilestoneResponderWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        let PeerManagerConfig {
            network_rx,
            peering_rx,
            network_name,
        } = config;

        if let Some(peering_rx) = peering_rx {
            node.spawn::<Self, _, _>(|shutdown| async move {
                use AutopeeringEvent::*;

                info!("Autopeering handler running.");

                let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(peering_rx));

                while let Some(event) = receiver.next().await {
                    trace!("Received event {:?}.", event);

                    match event {
                        IncomingPeering { peer, .. } => {
                            handle_new_peering(peer, &network_name, &network_command_tx);
                        }
                        OutgoingPeering { peer, .. } => {
                            handle_new_peering(peer, &network_name, &network_command_tx);
                        }
                        PeeringDropped { peer_id } => {
                            handle_peering_dropped(peer_id, &network_command_tx);
                        }
                        _ => {}
                    }
                }

                info!("Autopeering stopped.");
            });
        }

        node.spawn::<Self, _, _>(|shutdown| async move {
            use NetworkEvent::*;

            info!("Network handler running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(network_rx.into()));

            while let Some(event) = receiver.next().await {
                trace!("Received event {:?}.", event);

                match event {
                    PeerAdded { peer_id, info } => {
                        // TODO check if not already added ?
                        info!("Added peer {}.", info.alias);

                        let peer = Arc::new(Peer::new(peer_id, info));
                        peer_manager.add(peer).await;
                    }
                    PeerRemoved { peer_id } => {
                        if let Some(peer) = peer_manager.remove(&peer_id).await {
                            info!("Removed peer {}.", peer.0.alias());
                        }
                    }
                    PeerConnected {
                        peer_id,
                        info: _,
                        gossip_in: receiver,
                        gossip_out: sender,
                    } => {
                        // TODO write a get_mut peer manager method
                        if let Some(mut peer) = peer_manager.get_mut(&peer_id).await {
                            let (shutdown_tx, shutdown_rx) = oneshot::channel();

                            peer.0.set_connected(true);
                            peer.1 = Some((sender, shutdown_tx));

                            tokio::spawn(
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
                        send_heartbeat(
                            &*peer_manager,
                            &*metrics,
                            &peer_id,
                            new_heartbeat(&*peer_manager, &*tangle).await,
                        )
                        .await;
                    }
                    PeerDisconnected { peer_id } => {
                        // TODO write a get_mut peer manager method
                        if let Some(peer) = peer_manager.0.write().await.peers.get_mut(&peer_id) {
                            peer.0.set_connected(false);
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

            info!("Network stopped.");
        });

        Ok(Self {})
    }
}

fn handle_new_peering(peer: bee_autopeering::Peer, network_name: &str, command_tx: &NetworkCommandSender) {
    if let Some(multiaddr) = peer.get_service_multiaddr(network_name) {
        let peer_id = peer.peer_id().to_libp2p_peer_id();

        command_tx
            .send(Command::AddPeer {
                peer_id,
                alias: Some(alias!(peer_id).to_string()),
                multiaddr,
                relation: PeerRelation::Discovered,
            })
            .expect("error sending network command");
    }
}

fn handle_peering_dropped(peer_id: bee_autopeering::PeerId, command_tx: &NetworkCommandSender) {
    let peer_id = peer_id.to_libp2p_peer_id();

    command_tx
        .send(Command::RemovePeer { peer_id })
        .expect("error sending network command");
}
