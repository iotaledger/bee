// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use super::protocols::gossip::{self, Gossip, GossipEvent};
use crate::{
    alias,
    service::{SwarmEvent, SwarmEventSender},
};

use futures::AsyncReadExt;
use libp2p::{
    identify::{Identify, IdentifyEvent},
    identity::PublicKey,
    kad::{store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent},
    swarm::NetworkBehaviourEventProcess,
    Multiaddr, NetworkBehaviour, PeerId,
};
use log::*;

const KADEMLIA_QUERY_TIMEOUT: u64 = 1 * 60;
const IDENTIFY_PROTOCOL_NAME: &str = "iota/0.1.0";
const IDENTIFY_AGENT_VERSION: &str = "github.com/iotaledger/bee";

pub type Swarm = libp2p::swarm::Swarm<SubstreamBehavior>;

/// A type that contains the different protocols that are negotiated on top of the basic transport of a peer connection.
#[derive(NetworkBehaviour)]
pub struct SubstreamBehavior {
    /// Identify protocol.
    identify: Identify,

    /// IOTA gossip protocol.
    gossip: Gossip,

    /// Kademlia protocol.
    kademlia: Kademlia<MemoryStore>,

    #[behaviour(ignore)]
    swarm_event_sender: SwarmEventSender,
}

impl SubstreamBehavior {
    pub async fn new(local_public_key: PublicKey, swarm_event_sender: SwarmEventSender) -> Self {
        let local_id = local_public_key.clone().into();

        let peer_store = MemoryStore::new(local_id);
        let mut kad_config = KademliaConfig::default();

        kad_config.set_query_timeout(Duration::from_secs(KADEMLIA_QUERY_TIMEOUT));
        kad_config.disjoint_query_paths(true);

        let kademlia = Kademlia::with_config(local_id, peer_store, kad_config);

        info!(
            "Kademlia protocol name: {}",
            String::from_utf8_lossy(kademlia.protocol_name())
        );

        info!("Identify protocol name: {}", IDENTIFY_PROTOCOL_NAME);

        let identify = Identify::new(
            IDENTIFY_PROTOCOL_NAME.to_string(),
            IDENTIFY_AGENT_VERSION.to_string(),
            local_public_key,
        );

        Self {
            identify,
            gossip: Gossip::new(),
            kademlia,
            swarm_event_sender,
        }
    }

    /// Adds the address of a peer to the routing table.
    /// **Note**: Does nothing if Kademlia is disabled.
    pub fn add_address_to_routing_table(&mut self, peer_id: &PeerId, address: Multiaddr) {
        self.kademlia.add_address(peer_id, address);
    }

    ///
    pub fn bootstrap_local_routing_table(&mut self) -> bool {
        self.kademlia.bootstrap().is_ok()
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for SubstreamBehavior {
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

impl NetworkBehaviourEventProcess<GossipEvent> for SubstreamBehavior {
    fn inject_event(&mut self, event: GossipEvent) {
        trace!("Behavior received gossip event.");

        match event {
            GossipEvent::Success {
                peer_id,
                peer_addr,
                conn,
                conn_info,
            } => {
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
            GossipEvent::Failure => (),
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for SubstreamBehavior {
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
                    .send(SwarmEvent::RoutingTableUpdated {
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
