// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use super::protocols::gossip::{self, Gossip, GossipEvent};
use crate::{
    host::Origin,
    service::{InternalEvent, InternalEventSender},
};

use futures::AsyncReadExt;
use libp2p::{
    identify::{Identify, IdentifyEvent},
    identity::PublicKey,
    kad::{
        record::store::RecordStore, store::MemoryStore, GetClosestPeersError, Kademlia, KademliaConfig, KademliaEvent,
        QueryResult,
    },
    multiaddr::Protocol,
    swarm::{NetworkBehaviour, NetworkBehaviourEventProcess},
    Multiaddr, NetworkBehaviour, PeerId,
};
use log::*;
use tokio::sync::mpsc;

#[derive(NetworkBehaviour)]
pub struct SwarmBehavior {
    identify: Identify,
    gossip: Gossip,
    kad: Kademlia<MemoryStore>,
    #[behaviour(ignore)]
    internal_sender: InternalEventSender,
}

impl SwarmBehavior {
    pub async fn new(
        local_public_key: PublicKey,
        internal_sender: InternalEventSender,
        origin_rx: mpsc::UnboundedReceiver<Origin>,
        entry_nodes: Vec<Multiaddr>,
    ) -> Self {
        let local_id = local_public_key.clone().into();

        const QUERY_TIMEOUT: u64 = 1 * 60; // 5 mins
        let peer_store = MemoryStore::new(local_id);
        let mut kad_config = KademliaConfig::default();
        kad_config.set_query_timeout(Duration::from_secs(QUERY_TIMEOUT));

        let mut kad = Kademlia::with_config(local_id, peer_store, kad_config);

        for mut p2p_addr in entry_nodes {
            if let Some(Protocol::P2p(multihash)) = p2p_addr.pop() {
                kad.add_address(&PeerId::from_multihash(multihash).unwrap(), p2p_addr);
            }
        }

        Self {
            identify: Identify::new(
                "iota/0.1.0".to_string(),
                "github.com/iotaledger/bee".to_string(),
                local_public_key,
            ),
            gossip: Gossip::new(origin_rx),
            kad,
            internal_sender,
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

        gossip::spawn_gossip_in_task(peer_id, reader, incoming_gossip_sender, self.internal_sender.clone());
        gossip::spawn_gossip_out_task(peer_id, writer, outgoing_gossip_receiver, self.internal_sender.clone());

        // TODO: retrieve the PeerInfo from the peer list

        let _ = self
            .internal_sender
            .send(InternalEvent::ProtocolEstablished {
                peer_id,
                peer_addr,
                conn_info,
                gossip_in: incoming_gossip_receiver,
                gossip_out: outgoing_gossip_sender,
            })
            .expect("Receiver of internal event channel dropped.");
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::QueryResult { id, result, stats } => {
                println!("Kademlia QueryResult.");
            }
            KademliaEvent::RoutingUpdated {
                peer,
                addresses,
                old_peer,
            } => {
                println!("Kademlia RoutingUpdated.");
            }
            KademliaEvent::UnroutablePeer { peer } => {
                println!("Kademlia UnroutablePeer.");
            }
            KademliaEvent::RoutablePeer { peer, address } => {
                println!("Kademlia RoutablePeer.");
            }
            KademliaEvent::PendingRoutablePeer { peer, address } => {
                println!("Kademlia PendingRoutablePeer.");
            }
        }
    }
}
