// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{behavior::SwarmBehavior, error::Error};

use crate::service::event::InternalEventSender;

use bee_runtime::task::{StandaloneSpawner, TaskSpawner};

use libp2p::{
    core::{
        connection::ConnectionLimits,
        upgrade::{self, SelectUpgrade},
    },
    dns, identity, mplex, noise,
    swarm::SwarmBuilder,
    tcp, yamux, Swarm, Transport,
};

use std::time::Duration;

const MAX_CONNECTIONS_PER_PEER: u32 = 1;
const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;

pub fn build_swarm(
    local_keys: &identity::Keypair,
    internal_sender: InternalEventSender,
) -> Result<Swarm<SwarmBehavior>, Error> {
    let local_pk = local_keys.public();
    let local_id = local_pk.clone().into_peer_id();

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .map_err(|_| Error::CreatingNoiseKeysFailed)?;

    let noi_config = noise::NoiseConfig::xx(noise_keys);
    let mpx_config = mplex::MplexConfig::default();
    let ymx_config = yamux::YamuxConfig::default();

    let transport = if cfg!(test) {
        use libp2p_core::transport::MemoryTransport;

        MemoryTransport::default()
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noi_config.into_authenticated())
            .multiplex(SelectUpgrade::new(ymx_config, mpx_config))
            .timeout(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS))
            .boxed()
    } else {
        let tcp_config = tcp::TokioTcpConfig::new().nodelay(true).port_reuse(true);
        let dns_config = dns::TokioDnsConfig::system(tcp_config)?;

        dns_config
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noi_config.into_authenticated())
            .multiplex(SelectUpgrade::new(ymx_config, mpx_config))
            .timeout(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS))
            .boxed()
    };

    let behavior = SwarmBehavior::new(local_pk, internal_sender);
    let limits = ConnectionLimits::default().with_max_established_per_peer(Some(MAX_CONNECTIONS_PER_PEER));

    let swarm = SwarmBuilder::new(transport, behavior, local_id)
        .connection_limits(limits)
        // We want the connection background tasks to be spawned
        // onto the tokio runtime.
        .executor(Box::new(|fut| {
            StandaloneSpawner::spawn(fut);
        }))
        .build();

    Ok(swarm)
}
