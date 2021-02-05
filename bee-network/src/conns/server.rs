// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, NetworkService, PeerInfo, PeerList, PeerRelation},
    transport::build_transport,
    Multiaddr, PeerId, ShortId,
};

use super::{errors::Error, Origin};

use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::prelude::*;
use lazy_static::lazy_static;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::ListenerEvent},
    identity, Transport,
};
use log::*;

use std::{
    any::TypeId,
    convert::Infallible,
    io,
    pin::Pin,
    sync::{Arc, RwLock},
};

type ListenerUpgrade = Pin<Box<(dyn Future<Output = Result<(PeerId, StreamMuxerBox), io::Error>> + Send + 'static)>>;
type PeerListener = Pin<Box<dyn Stream<Item = Result<ListenerEvent<ListenerUpgrade, io::Error>, io::Error>> + Send>>;

lazy_static! {
    pub static ref LISTEN_ADDRESSES: Arc<RwLock<Vec<Multiaddr>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Default)]
pub struct Server {}

pub struct ServerConfig {
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    peer_listener: PeerListener,
    internal_event_sender: InternalEventSender,
}

impl ServerConfig {
    pub async fn new(
        local_keys: identity::Keypair,
        // TODO: allow multiple bind addresses
        bind_address: Multiaddr,
        peers: PeerList,
        banned_addrs: BannedAddrList,
        banned_peers: BannedPeerList,
        internal_event_sender: InternalEventSender,
    ) -> Result<Self, Error> {
        // Create underlying Tcp connection and negotiate Noise and Mplex/Yamux
        let transport = build_transport(&local_keys).map_err(|_| Error::CreatingTransportFailed)?;

        let mut peer_listener = transport
            .listen_on(bind_address.clone())
            .map_err(|_| Error::BindingAddressFailed(bind_address))?;

        if let Some(Ok(ListenerEvent::NewAddress(listen_address))) = peer_listener.next().await {
            LISTEN_ADDRESSES
                .write()
                .expect("adding listen address")
                .push(listen_address.clone());

            trace!("Accepting connections on {}.", listen_address);
        } else {
            return Err(Error::NotListeningError);
        }

        Ok(Self {
            peers,
            banned_peers,
            banned_addrs,
            peer_listener,
            internal_event_sender,
        })
    }
}

#[async_trait]
impl<N: Node> Worker<N> for Server {
    type Config = ServerConfig;
    type Error = Infallible;

    // NOTE: The server is dependent on the `NetworkService` as it processes the events produced by the server.
    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<NetworkService>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        info!("Server started.");

        let ServerConfig {
            peers,
            banned_peers,
            banned_addrs,
            peer_listener,
            internal_event_sender,
            ..
        } = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Listening for peers.");

            let mut incoming = ShutdownStream::new(shutdown, peer_listener);

            // NOTE: The first events will be `ListenerEvent::NewAddress`
            while let Some(Ok(listener_event)) = incoming.next().await {
                match listener_event {
                    ListenerEvent::AddressExpired(addr) => trace!("Address expired: {}", addr),
                    ListenerEvent::Error(e) => error!("Libp2p error. Cause: {:?}", e),
                    ListenerEvent::NewAddress(ref listen_address) => {
                        LISTEN_ADDRESSES
                            .write()
                            .expect("adding listen address")
                            .push(listen_address.clone());

                        trace!("Accepting connections on {}.", listen_address);
                    }
                    ListenerEvent::Upgrade {
                        upgrade,
                        local_addr: _, // to which listen address the peer connected
                        remote_addr,   // the address of the peer that wants to connect
                    } => {
                        // TODO: try again to move this block into its own function (beware: lifetime issues ahead!!!)

                        // Prevent accepting from banned addresses.
                        let peer_address_str = remote_addr.to_string();
                        if banned_addrs.contains(&peer_address_str) {
                            warn!("Ignoring peer. Cause: {} is banned.", peer_address_str);
                            continue;
                        }

                        let (peer_id, muxer) = match upgrade.await {
                            Ok(u) => u,
                            Err(_) => {
                                warn!("Ignoring peer. Cause: Handshake failed.");
                                continue;
                            }
                        };

                        // Prevent accepting duplicate connections.
                        if let Ok(connected) = peers.is(&peer_id, |_, state| state.is_connected()).await {
                            if connected {
                                warn!("Already connected to {}", peer_id);
                                continue;
                            }
                        }

                        // Prevent accepting banned peers.
                        if banned_peers.contains(&peer_id) {
                            warn!("Ignoring peer. Cause: {} is banned.", peer_id);
                            continue;
                        }

                        let peer_info = if let Ok(peer_info) = peers.get_info(&peer_id).await {
                            // If we have this peer id in our peerlist (but are not already connected to it),
                            // then we allow the connection.
                            peer_info
                        } else {
                            // We also allow for a certain number of unknown peers.
                            let peer_info = PeerInfo {
                                address: remote_addr,
                                alias: peer_id.short(),
                                relation: PeerRelation::Unknown,
                            };

                            if let Err(e) = peers.accepts(&peer_id, &peer_info).await {
                                warn!("Unknown peer rejected. Cause: {}.", e);
                                continue;
                            } else {
                                // We also allow for a certain number of unknown peers.
                                info!("Unknown peer '{}' accepted.", peer_info.alias);

                                peer_info
                            }
                        };

                        if let Err(e) = super::upgrade_connection(
                            peer_id.clone(),
                            peer_info,
                            muxer,
                            Origin::Inbound,
                            internal_event_sender.clone(),
                        )
                        .await
                        {
                            error!("Error occurred during upgrading the connection. Cause: {}", e);
                            continue;
                        }
                    }
                }
            }

            info!("Server stopped.")
        });

        Ok(Self::default())
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        Ok(())
    }
}
