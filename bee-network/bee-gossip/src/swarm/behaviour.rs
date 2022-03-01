// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::protocols::iota_gossip::{IotaGossipEvent, IotaGossipProtocol};

use crate::{
    alias,
    service::event::{InternalEvent, InternalEventSender},
};

use libp2p::{
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    identity::PublicKey,
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use log::*;

const IOTA_PROTOCOL_VERSION: &str = "iota/0.1.0";

#[derive(NetworkBehaviour)]
pub struct SwarmBehaviour {
    identify: Identify,
    gossip: IotaGossipProtocol,
    #[behaviour(ignore)]
    internal_sender: InternalEventSender,
}

impl SwarmBehaviour {
    pub fn new(local_pk: PublicKey, internal_sender: InternalEventSender) -> Self {
        let protocol_version = IOTA_PROTOCOL_VERSION.to_string();
        let config = IdentifyConfig::new(protocol_version, local_pk);

        Self {
            identify: Identify::new(config),
            gossip: IotaGossipProtocol::new(),
            internal_sender,
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for SwarmBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        match event {
            IdentifyEvent::Received { peer_id, info } => {
                trace!("Received Identify response from {}: {:?}.", alias!(peer_id), info,);

                // Panic:
                // Sending must not fail.
                self.internal_sender
                    .send(InternalEvent::PeerIdentified { peer_id })
                    .expect("send internal event");
            }
            IdentifyEvent::Sent { peer_id } => {
                trace!("Sent Identify request to {}.", alias!(peer_id));
            }
            IdentifyEvent::Pushed { peer_id } => {
                trace!("Pushed Identify request to {}.", alias!(peer_id));
            }
            IdentifyEvent::Error { peer_id, error } => {
                debug!("Identification error with {}: Cause: {:?}.", alias!(peer_id), error);

                // Panic:
                // Sending must not fail.
                self.internal_sender
                    .send(InternalEvent::PeerUnreachable { peer_id })
                    .expect("send internal event");
            }
        }
    }
}

impl NetworkBehaviourEventProcess<IotaGossipEvent> for SwarmBehaviour {
    fn inject_event(&mut self, event: IotaGossipEvent) {
        match event {
            IotaGossipEvent::ReceivedUpgradeRequest { from } => {
                trace!("Received IOTA gossip request from {}.", alias!(from));
            }
            IotaGossipEvent::SentUpgradeRequest { to } => {
                trace!("Sent IOTA gossip request to {}.", alias!(to));
            }
            IotaGossipEvent::UpgradeCompleted {
                peer_id,
                peer_addr,
                origin,
                substream,
            } => {
                trace!("Successfully negotiated IOTA gossip protocol with {}.", alias!(peer_id));

                self.internal_sender
                    .send(InternalEvent::ProtocolEstablished {
                        peer_id,
                        peer_addr,
                        origin,
                        substream,
                    })
                    .expect("send internal event");
            }
            IotaGossipEvent::UpgradeError { peer_id, error } => {
                debug!(
                    "IOTA gossip upgrade error with {}: Cause: {:?}.",
                    alias!(peer_id),
                    error
                );
            }
        }
    }
}
