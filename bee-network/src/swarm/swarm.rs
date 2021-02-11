use libp2p::{
    core::{
        connection::ConnectionLimits,
        upgrade::{self, SelectUpgrade},
    },
    dns, identity, mplex, noise,
    swarm::SwarmBuilder,
    tcp, yamux, Swarm, Transport,
};

use std::{io, time::Duration};

use crate::service::events::InternalEventSender;

use super::SwarmBehavior;

const MAX_CONNECTIONS_PER_PEER: u32 = 1;

pub async fn build_swarm(
    local_keys: &identity::Keypair,
    internal_sender: InternalEventSender,
) -> io::Result<Swarm<SwarmBehavior>> {
    let local_public_key = local_keys.public();
    let local_peer_id = local_public_key.clone().into_peer_id();

    // TODO: error propagation
    let nse_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .expect("error creating noise keys");

    let tcp_config = tcp::TokioTcpConfig::new().nodelay(true);
    let noi_config = noise::NoiseConfig::xx(nse_keys);
    let dns_config = dns::DnsConfig::new(tcp_config)?;
    let mpx_config = mplex::MplexConfig::default();
    let ymx_config = yamux::YamuxConfig::default();

    let transport = dns_config
        .upgrade(upgrade::Version::V1)
        .authenticate(noi_config.into_authenticated())
        .multiplex(SelectUpgrade::new(ymx_config, mpx_config))
        .outbound_timeout(Duration::from_secs(60))
        .inbound_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(10))
        .boxed();

    let behavior = SwarmBehavior::new(local_public_key, internal_sender).await;
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
