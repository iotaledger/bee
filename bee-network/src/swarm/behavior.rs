// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::protocols::gossip::{self, Gossip, GossipEvent};
use crate::{
    host::Origin,
    service::{InternalEvent, InternalEventSender},
};

use futures::AsyncReadExt;
use libp2p::{
    identify::{Identify, IdentifyEvent},
    identity::PublicKey,
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use log::*;
use tokio::sync::mpsc;

#[derive(NetworkBehaviour)]
pub struct SwarmBehavior {
    identify: Identify,
    gossip: Gossip,
    #[behaviour(ignore)]
    internal_sender: InternalEventSender,
}

impl SwarmBehavior {
    pub async fn new(
        local_public_key: PublicKey,
        internal_sender: InternalEventSender,
        origin_rx: mpsc::UnboundedReceiver<Origin>,
    ) -> Self {
        Self {
            identify: Identify::new(
                "iota/0.1.0".to_string(),
                "github.com/iotaledger/bee".to_string(),
                local_public_key,
            ),
            gossip: Gossip::new(origin_rx),
            internal_sender,
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: IdentifyEvent) {
        trace!("IdentifyEvent.");

        match event {
            IdentifyEvent::Received {
                peer_id,
                info: _,
                observed_addr,
            } => {
                debug!(
                    "Received Identify request from {}. Observed addresses: {:?}.",
                    peer_id, observed_addr
                );
            }
            IdentifyEvent::Sent { peer_id } => {
                debug!("Sent Identify request to {}.", peer_id);
            }
            IdentifyEvent::Error { peer_id, error } => {
                warn!("Identification error with {}: Cause: {:?}.", peer_id, error);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<GossipEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: GossipEvent) {
        trace!("GossipEvent.");

        let GossipEvent {
            peer_id,
            peer_addr,
            conn,
            conn_info,
        } = event;

        info!("New gossip stream with {} [conn_info: {:?}]", peer_id, conn_info);

        let (reader, writer) = conn.split();

        let (incoming_gossip_sender, incoming_gossip_receiver) = gossip::gossip_channel();
        let (outgoing_gossip_sender, outgoing_gossip_receiver) = gossip::gossip_channel();

        gossip::spawn_gossip_in_task(peer_id, reader, incoming_gossip_sender, self.internal_sender.clone());
        gossip::spawn_gossip_out_task(peer_id, writer, outgoing_gossip_receiver, self.internal_sender.clone());

        // TODO: retrieve the PeerInfo from the peer list

        if self
            .internal_sender
            .send(InternalEvent::ProtocolEstablished {
                peer_id,
                peer_addr,
                conn_info,
                gossip_in: incoming_gossip_receiver,
                gossip_out: outgoing_gossip_sender,
            })
            .is_err()
        {
            trace!("Receiver of internal event channel dropped.");
        }
    }
}
