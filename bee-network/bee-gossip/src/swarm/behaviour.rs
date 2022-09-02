// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    NetworkBehaviour,
};
use libp2p_core::identity::PublicKey;

use super::protocols::iota_gossip::{IotaGossipEvent, IotaGossipProtocol};

const IOTA_PROTOCOL_VERSION: &str = "iota/0.1.0";

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "SwarmBehaviourEvent")]
pub struct SwarmBehaviour {
    identify: Identify,
    gossip: IotaGossipProtocol,
}

impl SwarmBehaviour {
    pub fn new(local_pk: PublicKey) -> Self {
        let protocol_version = IOTA_PROTOCOL_VERSION.to_string();
        let config = IdentifyConfig::new(protocol_version, local_pk);

        Self {
            identify: Identify::new(config),
            gossip: IotaGossipProtocol::new(),
        }
    }
}

pub enum SwarmBehaviourEvent {
    Identify(IdentifyEvent),
    Gossip(IotaGossipEvent),
}

impl From<IdentifyEvent> for SwarmBehaviourEvent {
    fn from(event: IdentifyEvent) -> Self {
        SwarmBehaviourEvent::Identify(event)
    }
}

impl From<IotaGossipEvent> for SwarmBehaviourEvent {
    fn from(event: IotaGossipEvent) -> Self {
        SwarmBehaviourEvent::Gossip(event)
    }
}
