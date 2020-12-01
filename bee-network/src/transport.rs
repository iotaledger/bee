// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{upgrade, Boxed},
    },
    dns, identity, mplex, noise, tcp, PeerId, Transport,
};

use std::io;

pub fn build_transport(local_keys: &identity::Keypair) -> io::Result<Boxed<(PeerId, StreamMuxerBox)>> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .expect("error creating noise keys");

    let tcp = tcp::TokioTcpConfig::new().nodelay(true);
    let transport = dns::DnsConfig::new(tcp)?;

    Ok(transport
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        // .multiplex(SelectUpgrade::new(yamux::Config::default(), mplex::MplexConfig::new()))
        .timeout(std::time::Duration::from_secs(20))
        .boxed())
}
