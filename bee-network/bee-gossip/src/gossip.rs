// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides the [`Gossip`] type, which allows establish and maintain gossip connections with peers.

use crate::{
    config::GossipConfig,
    conn::{ConnectedList, Direction},
    event::GossipEvent,
    handshake::handshake,
    peer::ConnectedPeer,
};

use bee_identity::{config::IdentityConfig, LocalId};
use bee_peering_manual::config::ManualPeeringConfig;
use bee_task::{StandaloneSpawner, TaskSpawner as _};

use tokio::{
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};

use std::io;

const RECONNECT_INTERVAL_SECS: u64 = 30;

/// Gossip layer errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Binding a TCP listener to an address failed.
    #[error("binding to address failed")]
    BindingToAddr,
    /// Reading from a network socket failed.
    #[error("reading from socket failed: {0}")]
    SocketRead(io::Error),
    /// Writing to a network socket failed.
    #[error("writing to socket failed: {0}")]
    SocketWrite(io::Error),
}

/// A type that realizes and maintains the gossip communication layer in a node.
pub struct Gossip {}

impl Gossip {
    /// Starts the gossip (layer).
    pub async fn start(
        gossip_config: GossipConfig,
        identity_config: IdentityConfig,
        manual_peering_config: ManualPeeringConfig,
        on_event: impl Fn(GossipEvent) + Clone + Send + 'static,
    ) -> Result<(), Error> {
        let GossipConfig { bind_addr } = gossip_config;
        let IdentityConfig { local_id } = identity_config;

        let server = TcpListener::bind(bind_addr).await.map_err(|_| Error::BindingToAddr)?;
        log::info!("Listening for gossip at: {}", bind_addr);

        let connected_list = ConnectedList::new();

        // Spin up a server listening for peers.
        StandaloneSpawner::spawn(run_server(
            server,
            on_event.clone(),
            local_id.clone(),
            manual_peering_config.clone(),
            connected_list.clone(),
        ));

        // Spin up a client actively connecting to peers.
        StandaloneSpawner::spawn(run_client(on_event, local_id, manual_peering_config, connected_list));

        Ok(())
    }
}

async fn run_server(
    server: TcpListener,
    on_event: impl Fn(GossipEvent),
    local_id: LocalId,
    manual_peering_config: ManualPeeringConfig,
    connected_list: ConnectedList,
) {
    loop {
        let result = server.accept().await;
        match result {
            Ok((tcp_stream, socket_addr)) => {
                log::info!("being dialed from address {}", socket_addr);

                // check whether it's okay being dialed from that address.
                if connected_list.contains(socket_addr) {
                    log::info!("already connected to address {}", socket_addr);
                    continue;
                }

                if let Some(peer_info) = manual_peering_config.get(&socket_addr.ip()) {
                    match handshake(
                        tcp_stream,
                        socket_addr,
                        &local_id,
                        Direction::Inbound,
                        peer_info.clone(),
                    )
                    .await
                    {
                        Ok((reader, writer, identity, alias)) => {
                            connected_list.add(socket_addr);

                            let connected_peer = ConnectedPeer::new(identity, alias, reader, writer);

                            on_event(GossipEvent::PeerConnected(connected_peer));
                        }
                        Err(e) => {
                            log::warn!("handshake error {:?} with {}", e, socket_addr);
                        }
                    }
                } else {
                    log::warn!("address denied: {}", socket_addr.ip());
                }
            }
            Err(e) => {
                log::warn!("{}", e);
            }
        }
    }
}

// TODO: realise when a connected peer becomes unhealthy, and allow reconnection!
async fn run_client(
    on_event: impl Fn(GossipEvent),
    local_id: LocalId,
    manual_peering_config: ManualPeeringConfig,
    connected_list: ConnectedList,
) {
    loop {
        for (_ip, peer_info) in manual_peering_config.iter() {
            // continue, if:
            // (a) the peer is already connected, or
            // (b) the peer is supposed to dial *us*.
            if connected_list.contains(peer_info.address) || peer_info.is_dialer() {
                continue;
            } else {
                let socket_addr = peer_info.address;
                let result = TcpStream::connect(socket_addr).await;

                match result {
                    Ok(tcp_stream) => {
                        log::info!("dialing address: {}...", socket_addr);

                        match handshake(
                            tcp_stream,
                            socket_addr,
                            &local_id,
                            Direction::Outbound,
                            peer_info.clone(),
                        )
                        .await
                        {
                            Ok((reader, writer, identity, alias)) => {
                                connected_list.add(socket_addr);

                                let connected_peer = ConnectedPeer::new(identity, alias, reader, writer);

                                on_event(GossipEvent::PeerConnected(connected_peer))
                            }
                            Err(e) => {
                                log::warn!("handshake error {:?} with {}", e, socket_addr);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("{}", e);
                    }
                }
            }
        }

        sleep(Duration::from_secs(RECONNECT_INTERVAL_SECS)).await;
    }
}
