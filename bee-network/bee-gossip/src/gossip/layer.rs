// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use super::behaviour::{Gossip, GossipEvent};
use crate::peer::peer_id::PeerId;

use libp2p::{
    core::{
        connection::ConnectionLimits,
        upgrade::{self, SelectUpgrade},
    },
    dns,
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    mplex, noise,
    ping::{Ping, PingConfig, PingEvent},
    swarm::SwarmBuilder,
    tcp, yamux, NetworkBehaviour, Swarm, Transport,
};
use libp2p_core::identity;

use std::time::Duration;

const GOSSIP_PROTOCOL_NAME: &str = "iota-gossip";
const GOSSIP_VERSION: &str = "1.0.0";
const IOTA_PROTOCOL_VERSION: &str = "iota/0.1.0";

const CONNECTION_TIMEOUT_DEFAULT: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS_WITH_PEER: u32 = 1;
const NO_DELAY: bool = true;
const PORT_REUSE: bool = true;
const PING_KEEP_ALIVE: bool = false;

pub(crate) type GossipLayer = Swarm<GossipLayerBehaviour>;

#[derive(Debug)]
pub(crate) enum GossipLayerEvent {
    Identify(IdentifyEvent),
    Ping(PingEvent),
    Gossip(GossipEvent),
}

impl From<IdentifyEvent> for GossipLayerEvent {
    fn from(event: IdentifyEvent) -> Self {
        Self::Identify(event)
    }
}

impl From<PingEvent> for GossipLayerEvent {
    fn from(event: PingEvent) -> Self {
        Self::Ping(event)
    }
}

impl From<GossipEvent> for GossipLayerEvent {
    fn from(event: GossipEvent) -> Self {
        Self::Gossip(event)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GossipLayerError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Creating Noise authentication keys failed")]
    NoiseKeys,
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "GossipLayerEvent")]
pub(crate) struct GossipLayerBehaviour {
    identify: Identify,
    ping: Ping,
    gossip: Gossip,
}

impl GossipLayerBehaviour {
    pub fn new(local_public_key: identity::PublicKey, network_id: u64) -> Self {
        let protocol_version = IOTA_PROTOCOL_VERSION.to_string();
        let identify_config = IdentifyConfig::new(protocol_version, local_public_key);

        let ping_config = PingConfig::new().with_keep_alive(PING_KEEP_ALIVE);

        let gossip_network_name: &'static str =
            Box::leak(format!("/{GOSSIP_PROTOCOL_NAME}/{network_id}/{GOSSIP_VERSION}").into_boxed_str());

        Self {
            identify: Identify::new(identify_config),
            ping: Ping::new(ping_config),
            gossip: Gossip::new(gossip_network_name),
        }
    }
}

pub(crate) fn init_gossip_layer(
    local_keys: identity::Keypair,
    local_peer_id: PeerId,
    network_id: u64,
) -> Result<GossipLayer, GossipLayerError> {
    let local_public_key = local_keys.public();

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_keys)
        .map_err(|_| GossipLayerError::NoiseKeys)?;

    let noise_config = noise::NoiseConfig::xx(noise_keys);
    let mplex_config = mplex::MplexConfig::default();
    let yamux_config = yamux::YamuxConfig::default();

    let transport_layer = if cfg!(test) {
        use libp2p_core::transport::MemoryTransport;

        MemoryTransport::default()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise_config.into_authenticated())
            .multiplex(SelectUpgrade::new(yamux_config, mplex_config))
            .timeout(CONNECTION_TIMEOUT_DEFAULT)
            .boxed()
    } else {
        let tcp_config = tcp::TokioTcpConfig::new().nodelay(NO_DELAY).port_reuse(PORT_REUSE);
        let dns_config = dns::TokioDnsConfig::system(tcp_config)?;

        dns_config
            .upgrade(upgrade::Version::V1)
            .authenticate(noise_config.into_authenticated())
            .multiplex(SelectUpgrade::new(yamux_config, mplex_config))
            .timeout(CONNECTION_TIMEOUT_DEFAULT)
            .boxed()
    };

    let gossip_layer_behaviour = GossipLayerBehaviour::new(local_public_key, network_id);
    let limits = ConnectionLimits::default().with_max_established_per_peer(Some(MAX_CONNECTIONS_WITH_PEER));

    let gossip_layer = SwarmBuilder::new(transport_layer, gossip_layer_behaviour, *local_peer_id)
        .connection_limits(limits)
        // We want the connection background tasks to be spawned
        // onto the tokio runtime.
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    Ok(gossip_layer)
}
