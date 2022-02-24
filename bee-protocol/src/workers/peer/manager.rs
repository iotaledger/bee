// Copyright 2020-2022 IOTA Stiftung
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
use bee_gossip::{
    GossipManager, GossipManagerCommand, GossipManagerCommandTx, GossipManagerEvent, GossipManagerEventRx,
    PeerRelation, PeerType,
};
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};

use async_trait::async_trait;
use futures::{channel::oneshot, StreamExt};
use log::{info, trace, warn};
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible, sync::Arc};

pub(crate) struct PeerManagerConfig {
    pub(crate) gossip_event_rx: GossipManagerEventRx,
    pub(crate) peering_event_rx: Option<AutopeeringEventRx>,
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
            TypeId::of::<GossipManager>(),
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
        let gossip_manager_command_tx = node.resource::<GossipManagerCommandTx>();

        let hasher = node.worker::<HasherWorker>().unwrap().tx.clone();
        let message_responder = node.worker::<MessageResponderWorker>().unwrap().tx.clone();
        let milestone_responder = node.worker::<MilestoneResponderWorker>().unwrap().tx.clone();
        let milestone_requester = node.worker::<MilestoneRequesterWorker>().unwrap().tx.clone();

        let PeerManagerConfig {
            gossip_event_rx,
            peering_event_rx,
            network_name,
        } = config;

        if let Some(peering_rx) = peering_event_rx {
            node.spawn::<Self, _, _>(|shutdown| async move {
                info!("Autopeering handler running.");

                let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(peering_rx));

                while let Some(event) = receiver.next().await {
                    trace!("Received event {:?}.", event);

                    match event {
                        AutopeeringEvent::IncomingPeering { peer, .. } => {
                            handle_new_peering(peer, &network_name, &gossip_manager_command_tx);
                        }
                        AutopeeringEvent::OutgoingPeering { peer, .. } => {
                            handle_new_peering(peer, &network_name, &gossip_manager_command_tx);
                        }
                        AutopeeringEvent::PeeringDropped { peer_id } => {
                            handle_peering_dropped(peer_id, &gossip_manager_command_tx);
                        }
                        _ => {}
                    }
                }

                info!("Autopeering handler stopped.");
            });
        }

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Network handler running.");

            let mut gossip_events = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(gossip_event_rx));

            while let Some(gossip_event) = gossip_events.next().await {
                trace!("Received gossip event {gossip_event:?}.");

                match gossip_event {
                    GossipManagerEvent::PeerConnected {
                        peer_id,
                        peer_info,
                        writer: sender,
                        reader: receiver,
                    } => {
                        // TODO check if not already added ?
                        info!("Added peer {}.", peer_info.alias);

                        let peer = Arc::new(Peer::new(peer_id, peer_info));
                        peer_manager.add(peer).await;

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
                            &new_heartbeat(&*tangle, &*peer_manager).await,
                            &peer_id,
                            &*peer_manager,
                            &*metrics,
                        )
                        .await;
                    }
                    GossipManagerEvent::PeerDisconnected { peer_id } => {
                        if let Some(mut peer) = peer_manager.get_mut(&peer_id).await {
                            peer.0.set_connected(false);
                            if let Some((_, shutdown)) = peer.1.take() {
                                if let Err(e) = shutdown.send(()) {
                                    warn!("Sending shutdown to {} failed: {:?}.", peer.0.alias(), e);
                                }
                            }
                            info!("Disconnected peer {}.", peer.0.alias());
                        }

                        if let Some(peer) = peer_manager.remove(&peer_id).await {
                            info!("Removed peer {}.", peer.0.alias());
                        }
                    }
                    GossipManagerEvent::PeerUnreachable {
                        peer_id: _,
                        peer_relation,
                    } => {
                        if peer_relation == PeerRelation::Discovered {
                            // TODO: tell the autopeering to remove that peer from the neighborhood
                        }
                    }
                }
            }

            info!("Network handler stopped.");
        });

        Ok(Self {})
    }
}

fn handle_new_peering(peer: bee_autopeering::Peer, network_name: &str, command_tx: &GossipManagerCommandTx) {
    if let Some(multiaddr) = peer.service_multiaddr(network_name) {
        let peer_id = peer.peer_id().libp2p_peer_id().into();

        command_tx
            .send(GossipManagerCommand::AddPeer {
                peer_id,
                peer_addr: multiaddr,
                peer_alias: Some(peer_id.to_string()),
                peer_type: PeerType::Auto,
            })
            .expect("send add-peer command");
    }
}

fn handle_peering_dropped(peer_id: bee_autopeering::PeerId, command_tx: &GossipManagerCommandTx) {
    let peer_id = peer_id.libp2p_peer_id().into();

    command_tx
        .send(GossipManagerCommand::RemovePeer { peer_id })
        .expect("send remove-peer command");
}
