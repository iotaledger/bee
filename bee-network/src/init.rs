// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use super::{
    config::NetworkConfig,
    error::Error,
    peer::list::{PeerList, PeerListWrapper},
    service::{
        command::command_channel,
        controller::{NetworkCommandSender, NetworkEventReceiver},
        event::{event_channel, Event, InternalEvent},
    },
    types::{PeerInfo, PeerRelation},
    Keypair, PeerId,
};

use crate::{
    alias,
    network::host::NetworkHostConfig,
    service::service::{self, NetworkServiceConfig},
    swarm::builder::build_swarm,
};

use libp2p::{identity, Swarm};
use log::info;
use once_cell::sync::OnceCell;

pub mod global {
    use super::*;

    static RECONNECT_INTERVAL_SECS: OnceCell<u64> = OnceCell::new();
    static NETWORK_ID: OnceCell<u64> = OnceCell::new();
    static MAX_UNKNOWN_PEERS: OnceCell<usize> = OnceCell::new();

    pub fn set_reconnect_interval_secs(reconnect_interval_secs: u64) {
        if cfg!(test) {
            let _ = RECONNECT_INTERVAL_SECS.set(reconnect_interval_secs);
        } else {
            RECONNECT_INTERVAL_SECS
                .set(reconnect_interval_secs)
                .expect("oncecell set");
        }
    }
    pub fn reconnect_interval_secs() -> u64 {
        *RECONNECT_INTERVAL_SECS.get().expect("oncecell get")
    }

    pub fn set_network_id(network_id: u64) {
        if cfg!(test) {
            let _ = NETWORK_ID.set(network_id);
        } else {
            NETWORK_ID.set(network_id).expect("oncecell set");
        }
    }
    pub fn network_id() -> u64 {
        *NETWORK_ID.get().expect("oncecell get")
    }

    pub fn set_max_unknown_peers(max_unknown_peers: usize) {
        if cfg!(test) {
            let _ = MAX_UNKNOWN_PEERS.set(max_unknown_peers);
        } else {
            MAX_UNKNOWN_PEERS.set(max_unknown_peers).expect("oncecell set");
        }
    }
    pub fn max_unknown_peers() -> usize {
        *MAX_UNKNOWN_PEERS.get().expect("oncecell get")
    }
}

/// Initializes a "standalone" version of the network layer.
pub mod standalone {
    use super::*;
    use crate::{network::host::standalone::NetworkHost, service::service::standalone::NetworkService};

    use futures::channel::oneshot;

    use std::future::Future;

    /// Initializes the network.
    pub async fn init(
        config: NetworkConfig,
        keys: Keypair,
        network_id: u64,
        shutdown: impl Future + Send + Unpin + 'static,
    ) -> Result<(NetworkCommandSender, NetworkEventReceiver), Error> {
        let (network_config, service_config, network_command_sender, network_event_receiver) =
            super::init(config, keys, network_id)?;

        let (shutdown_signal_tx1, shutdown_signal_rx1) = oneshot::channel::<()>();
        let (shutdown_signal_tx2, shutdown_signal_rx2) = oneshot::channel::<()>();

        tokio::spawn(async move {
            shutdown.await;

            shutdown_signal_tx1.send(()).expect("sending shutdown signal");
            shutdown_signal_tx2.send(()).expect("sending shutdown signal");
        });

        NetworkService::new(shutdown_signal_rx1).start(service_config).await;
        NetworkHost::new(shutdown_signal_rx2).start(network_config).await;

        Ok((network_command_sender, network_event_receiver))
    }
}

/// Initializes an "integrated" version of the network layer, which is used in the Bee node.
pub mod integrated {
    use super::*;
    use crate::{network::host::integrated::NetworkHost, service::service::integrated::NetworkService};

    use bee_runtime::node::{Node, NodeBuilder};

    /// Initializes the network.
    pub async fn init<N: Node>(
        config: NetworkConfig,
        keys: Keypair,
        network_id: u64,
        mut node_builder: N::Builder,
    ) -> Result<(N::Builder, NetworkEventReceiver), Error> {
        let (host_config, service_config, network_command_sender, network_event_receiver) =
            super::init(config, keys, network_id)?;

        node_builder = node_builder
            .with_worker_cfg::<NetworkHost>(host_config)
            .with_worker_cfg::<NetworkService>(service_config)
            .with_resource(network_command_sender);

        Ok((node_builder, network_event_receiver))
    }
}

fn init(
    config: NetworkConfig,
    keys: Keypair,
    network_id: u64,
) -> Result<
    (
        NetworkHostConfig,
        NetworkServiceConfig,
        NetworkCommandSender,
        NetworkEventReceiver,
    ),
    Error,
> {
    let NetworkConfig {
        bind_multiaddr,
        reconnect_interval_secs,
        max_unknown_peers,
        static_peers: peers,
    } = config;

    global::set_reconnect_interval_secs(reconnect_interval_secs);
    global::set_network_id(network_id);
    global::set_max_unknown_peers(max_unknown_peers);

    let (command_sender, command_receiver) = command_channel();
    let (internal_command_sender, internal_command_receiver) = command_channel();

    let (event_sender, event_receiver) = event_channel::<Event>();
    let (internal_event_sender, internal_event_receiver) = event_channel::<InternalEvent>();

    let local_keys = identity::Keypair::Ed25519(keys);
    let local_id = PeerId::from_public_key(local_keys.public());

    info!("Local Id: {}", local_id);

    event_sender
        .send(Event::LocalIdCreated { local_id })
        .map_err(|_| Error::LocalIdAnnouncementFailed)?;

    let peerlist = PeerListWrapper::new(PeerList::from_peers(local_id, peers.iter().cloned().collect()));

    for peer in peers.into_iter() {
        let peer_id = peer.peer_id;
        event_sender
            .send(Event::PeerAdded {
                peer_id,
                info: PeerInfo {
                    address: peer.multiaddr,
                    alias: peer.alias.unwrap_or_else(|| alias!(peer_id).into()),
                    relation: PeerRelation::Known,
                },
            })
            .map_err(|_| Error::StaticPeersAnnouncementFailed)?;
    }

    // Create the transport layer
    let mut swarm =
        build_swarm(&local_keys, internal_event_sender.clone()).map_err(|_| Error::CreatingTransportFailed)?;

    // Try binding to the configured bind address.
    info!("Binding to: {}", bind_multiaddr);
    let _listener_id = Swarm::listen_on(&mut swarm, bind_multiaddr.clone()).map_err(|_| Error::BindingAddressFailed)?;

    let host_config = NetworkHostConfig {
        internal_event_sender: internal_event_sender.clone(),
        internal_command_receiver,
        peerlist: peerlist.clone(),
        swarm,
    };

    let service_config = NetworkServiceConfig {
        local_keys,
        senders: service::Senders {
            events: event_sender,
            internal_events: internal_event_sender,
            internal_commands: internal_command_sender,
        },
        receivers: service::Receivers {
            commands: command_receiver,
            internal_events: internal_event_receiver,
        },
        peerlist,
    };

    let network_command_sender = NetworkCommandSender::new(command_sender);
    let network_event_receiver = NetworkEventReceiver::new(event_receiver);

    Ok((
        host_config,
        service_config,
        network_command_sender,
        network_event_receiver,
    ))
}
