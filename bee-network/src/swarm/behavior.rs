// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use super::protocols::gossip::{self, Gossip, GossipEvent};
use crate::{
    alias,
    host::Origin,
    service::{SwarmEvent, SwarmEventSender},
};

use futures::AsyncReadExt;
use libp2p::{
    identify::{Identify, IdentifyEvent},
    identity::PublicKey,
    kad::{store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent},
    multiaddr::Protocol,
    swarm::NetworkBehaviourEventProcess,
    Multiaddr, NetworkBehaviour, PeerId,
};
use log::*;
use tokio::sync::mpsc;

const KADEMLIA_QUERY_TIMEOUT: u64 = 1 * 60;

/// A type that contains the different protocols that are negotiated on top of the basic transport of a peer connection.
#[derive(NetworkBehaviour)]
pub struct SwarmBehavior {
    /// Identify protocol.
    pub(crate) identify: Identify,

    /// IOTA gossip protocol.
    pub(crate) gossip: Gossip,

    /// Kademlia protocol.
    pub(crate) kademlia: Kademlia<MemoryStore>,

    #[behaviour(ignore)]
    swarm_event_sender: SwarmEventSender,
}

impl SwarmBehavior {
    pub async fn new(
        local_public_key: PublicKey,
        swarm_event_sender: SwarmEventSender,
        origin_rx: mpsc::UnboundedReceiver<Origin>,
        entry_nodes: Vec<Multiaddr>,
    ) -> Self {
        let local_id = local_public_key.clone().into();

        let peer_store = MemoryStore::new(local_id);
        let mut kad_config = KademliaConfig::default();

        kad_config.set_query_timeout(Duration::from_secs(KADEMLIA_QUERY_TIMEOUT));
        kad_config.disjoint_query_paths(true);

        let mut kademlia = Kademlia::with_config(local_id, peer_store, kad_config);

        // Connect this node to the entry nodes
        for mut p2p_addr in entry_nodes {
            if let Some(Protocol::P2p(multihash)) = p2p_addr.pop() {
                kademlia.add_address(&PeerId::from_multihash(multihash).unwrap(), p2p_addr);
            }
        }

        Self {
            identify: Identify::new(
                "iota/0.1.0".to_string(),
                "github.com/iotaledger/bee".to_string(),
                local_public_key,
            ),
            gossip: Gossip::new(origin_rx),
            kademlia,
            swarm_event_sender,
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: IdentifyEvent) {
        trace!("Behavior received identify event.");

        match event {
            IdentifyEvent::Received {
                peer_id,
                info: _,
                observed_addr,
            } => {
                trace!(
                    "Received Identify request from {}. Observed addresses: {:?}.",
                    peer_id,
                    observed_addr
                );
            }
            IdentifyEvent::Sent { peer_id } => {
                trace!("Sent Identify request to {}.", peer_id);
            }
            IdentifyEvent::Error { peer_id, error } => {
                warn!("Identification error with {}: Cause: {:?}.", peer_id, error);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<GossipEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: GossipEvent) {
        trace!("Behavior received gossip event.");

        let GossipEvent {
            peer_id,
            peer_addr,
            conn,
            conn_info,
        } = event;

        debug!("New gossip stream with {} [conn_info: {:?}]", peer_id, conn_info);

        let (reader, writer) = conn.split();

        let (incoming_gossip_sender, incoming_gossip_receiver) = gossip::gossip_channel();
        let (outgoing_gossip_sender, outgoing_gossip_receiver) = gossip::gossip_channel();

        gossip::spawn_gossip_in_task(peer_id, reader, incoming_gossip_sender, self.swarm_event_sender.clone());
        gossip::spawn_gossip_out_task(
            peer_id,
            writer,
            outgoing_gossip_receiver,
            self.swarm_event_sender.clone(),
        );

        // TODO: retrieve the PeerInfo from the peer list

        let _ = self
            .swarm_event_sender
            .send(SwarmEvent::ProtocolEstablished {
                peer_id,
                address: peer_addr,
                conn_info,
                gossip_in: incoming_gossip_receiver,
                gossip_out: outgoing_gossip_sender,
            })
            .expect("Receiver of event channel dropped.");
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::QueryResult { id, result, stats } => {
                println!("Kademlia QueryResult.");
                println!("QueryId: {:?}", id);
                println!("QueryResult: {:?}", result);
                println!("QueryStats: {:?}", stats);
            }
            // Fired, when we updated the routing table via `add_address`
            KademliaEvent::RoutingUpdated {
                peer,
                addresses,
                old_peer,
            } => {
                println!("Kademlia RoutingUpdated.");
                println!("Peer: {}", alias!(peer));
                println!("Addresses: {:?}", addresses);
                println!("Old peer: {:?}", old_peer);

                let _ = self
                    .swarm_event_sender
                    .send(SwarmEvent::PeerDiscovered {
                        peer_id: peer,
                        addresses,
                    })
                    .expect("Receiver of event channel dropped.");
            }
            KademliaEvent::UnroutablePeer { peer } => {
                println!("Kademlia UnroutablePeer: {}.", alias!(peer));
            }
            KademliaEvent::RoutablePeer { peer, address } => {
                println!("Kademlia RoutablePeer: {} [{}].", alias!(peer), address);
            }
            KademliaEvent::PendingRoutablePeer { peer, address } => {
                println!("Kademlia PendingRoutablePeer: {} [{}].", peer, address);
            }
        }
    }
}
