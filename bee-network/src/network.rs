// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides the [`Network`] type, which allows establish and maintain network connections with peers.

use crate::{
    config::{Config, ManualPeerConfig},
    conn::{ConnectedList, Direction},
    event::NetworkEvent,
    handshake::handshake,
    identity::LocalIdentity,
    peer::ConnectedPeer,
};

use tokio::{
    net::{TcpListener, TcpStream},
    task::spawn,
    time::{sleep, Duration},
};

use std::sync::atomic::AtomicUsize;

static _NUM_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

const RECONNECT_INTERVAL_SECS: u64 = 30;

use std::io;

/// Network errors.
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

/// A type representing a network layer in order to establish and maintain connections with peers.
pub struct Network {}

impl Network {
    /// Starts the network (layer).
    pub async fn start(config: Config, on_event: impl Fn(NetworkEvent) + Clone + Send + 'static) -> Result<(), Error> {
        let Config {
            bind_addr,
            local_id,
            manual_peer_config,
        } = config;

        let server = TcpListener::bind(bind_addr).await.map_err(|_| Error::BindingToAddr)?;

        let connected_list = ConnectedList::new();

        // Spin up a server listening for peers.
        spawn(run_server(
            server,
            on_event.clone(),
            local_id.clone(),
            manual_peer_config.clone(),
            connected_list.clone(),
        ));

        // Spin up a client actively connecting to peers.
        spawn(run_client(on_event, local_id, manual_peer_config, connected_list));

        Ok(())
    }
}

async fn run_server(
    server: TcpListener,
    on_event: impl Fn(NetworkEvent),
    local_id: LocalIdentity,
    manual_peer_config: ManualPeerConfig,
    connected_list: ConnectedList,
) {
    loop {
        let result = server.accept().await;
        match result {
            Ok((tcp_stream, socket_addr)) => {
                // let conn_id = NUM_CONNECTIONS.fetch_add(1, Ordering::Relaxed);
                log::info!("being dialed from address {}", socket_addr);

                // check whether it's okay being dialed from that address.
                if connected_list.contains(socket_addr) {
                    log::info!("already connected to address {}", socket_addr);
                    continue;
                }

                if let Some(peer_info) = manual_peer_config.get(&socket_addr.ip()) {
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

                            on_event(NetworkEvent::PeerConnected(connected_peer));
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
    on_event: impl Fn(NetworkEvent),
    local_id: LocalIdentity,
    manual_peer_config: ManualPeerConfig,
    connected_list: ConnectedList,
) {
    loop {
        for (_ip, peer_info) in manual_peer_config.iter() {
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
                        // let conn_id = NUM_CONNECTIONS.fetch_add(1, Ordering::Relaxed);

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

                                on_event(NetworkEvent::PeerConnected(connected_peer))
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
