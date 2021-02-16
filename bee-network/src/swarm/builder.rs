// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::host::Origin;

use libp2p::{
    core::{
        connection::ConnectionLimits,
        upgrade::{self, SelectUpgrade},
    },
    dns, identity, mplex, noise,
    swarm::SwarmBuilder,
    tcp, yamux, Swarm, Transport,
};
use tokio::sync::mpsc::unbounded_channel;

use std::{io, time::Duration};

use crate::service::InternalEventSender;

use super::SwarmBehavior;

const MAX_CONNECTIONS_PER_PEER: u32 = 1;
const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 10;

pub async fn build_swarm(
    local_keys: &identity::Keypair,
    internal_sender: InternalEventSender,
) -> io::Result<Swarm<SwarmBehavior>> {
    let local_public_key = local_keys.public();
    let local_peer_id = local_public_key.clone().into_peer_id();

    // TODO: error propagation
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .expect("error creating noise keys");

    let tcp_config = tcp::TokioTcpConfig::new().nodelay(true).port_reuse(true);
    let noi_config = noise::NoiseConfig::xx(noise_keys);
    let dns_config = dns::DnsConfig::new(tcp_config)?;
    let mpx_config = mplex::MplexConfig::default();
    let ymx_config = yamux::YamuxConfig::default();

    let transport = dns_config
        .upgrade(upgrade::Version::V1)
        .authenticate(noi_config.into_authenticated())
        .multiplex(SelectUpgrade::new(ymx_config, mpx_config))
        .timeout(Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS))
        .boxed();

    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel::<Origin>();
    let behavior = SwarmBehavior::new(local_public_key, internal_sender, rx).await;
    let limits = ConnectionLimits::default().with_max_established_per_peer(Some(MAX_CONNECTIONS_PER_PEER));

    let swarm = SwarmBuilder::new(transport, behavior, local_peer_id)
        .connection_limits(limits)
        // We want the connection background tasks to be spawned
        // onto the tokio runtime.
        .executor(Box::new(|fut| {
            tokio::spawn(fut);
        }))
        .build();

    Ok(swarm)
}
