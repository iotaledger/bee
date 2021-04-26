// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::protocols::gossip::{self, Gossip, GossipEvent};

use crate::{
    alias,
    network::meta::Origin,
    service::event::{InternalEvent, InternalEventSender},
};

use futures::AsyncReadExt;
use libp2p::{
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    identity::PublicKey,
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use log::*;
use tokio::sync::mpsc;

const IOTA_PROTOCOL_VERSION: &str = "iota/0.1.0";

#[derive(NetworkBehaviour)]
pub struct SwarmBehavior {
    identify: Identify,
    gossip: Gossip,
    #[behaviour(ignore)]
    internal_sender: InternalEventSender,
}

impl SwarmBehavior {
    pub async fn new(
        local_pk: PublicKey,
        internal_sender: InternalEventSender,
        origin_rx: mpsc::UnboundedReceiver<Origin>,
    ) -> Self {
        let protocol_version = IOTA_PROTOCOL_VERSION.to_string();
        let config = IdentifyConfig::new(protocol_version, local_pk);

        Self {
            identify: Identify::new(config),
            gossip: Gossip::new(origin_rx),
            internal_sender,
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: IdentifyEvent) {
        trace!("Behavior received identify event.");

        match event {
            IdentifyEvent::Received { peer_id, info } => {
                trace!(
                    "Received Identify request from {}. Observed address: {:?}.",
                    alias!(peer_id),
                    info.observed_addr,
                );
            }
            IdentifyEvent::Sent { peer_id } => {
                trace!("Sent Identify request to {}.", alias!(peer_id));
            }
            IdentifyEvent::Pushed { peer_id } => {
                trace!("Pushed Identify request to {}.", alias!(peer_id));
            }
            IdentifyEvent::Error { peer_id, error } => {
                warn!("Identification error with {}: Cause: {:?}.", alias!(peer_id), error);
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

        debug!("New gossip stream with {}.", alias!(peer_id));

        let (reader, writer) = conn.split();

        let (incoming_gossip_sender, incoming_gossip_receiver) = gossip::gossip_channel();
        let (outgoing_gossip_sender, outgoing_gossip_receiver) = gossip::gossip_channel();

        gossip::spawn_gossip_in_processor(peer_id, reader, incoming_gossip_sender, self.internal_sender.clone());
        gossip::spawn_gossip_out_processor(peer_id, writer, outgoing_gossip_receiver, self.internal_sender.clone());

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
