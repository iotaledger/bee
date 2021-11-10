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
pub struct SwarmBehavior {
    identify: Identify,
    gossip: IotaGossipProtocol,
    #[behaviour(ignore)]
    internal_sender: InternalEventSender,
}

impl SwarmBehavior {
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

impl NetworkBehaviourEventProcess<IdentifyEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: IdentifyEvent) {
        match event {
            IdentifyEvent::Received { peer_id, info } => {
                trace!(
                    "Received Identify request from {}. Observed address: {:?}.",
                    alias!(peer_id),
                    info.observed_addr,
                );

                // TODO: log supported protocols by the peer (info.protocols)
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

impl NetworkBehaviourEventProcess<IotaGossipEvent> for SwarmBehavior {
    fn inject_event(&mut self, event: IotaGossipEvent) {
        match event {
            IotaGossipEvent::ReceivedUpgradeRequest { from } => {
                debug!("Received IOTA gossip request from {}.", alias!(from));
            }
            IotaGossipEvent::SentUpgradeRequest { to } => {
                debug!("Sent IOTA gossip request to {}.", alias!(to));
            }
            IotaGossipEvent::UpgradeCompleted {
                peer_id,
                peer_addr,
                origin,
                substream,
            } => {
                debug!("Successfully negotiated IOTA gossip protocol with {}.", alias!(peer_id));

                if let Err(e) = self.internal_sender.send(InternalEvent::ProtocolEstablished {
                    peer_id,
                    peer_addr,
                    origin,
                    substream,
                }) {
                    warn!(
                        "Send event error for {} after successfully established IOTA gossip protocol. Cause: {}",
                        peer_id, e
                    );

                    // TODO: stop processors in that case.
                }
            }
            IotaGossipEvent::UpgradeError { peer_id, error } => {
                warn!(
                    "IOTA gossip upgrade error with {}: Cause: {:?}.",
                    alias!(peer_id),
                    error
                );
            }
        }
    }
}
