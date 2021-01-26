// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    interaction::events::InternalEventSender,
    peers::{BannedAddrList, BannedPeerList, PeerInfo, PeerList, PeerRelation},
    transport::build_transport,
    Multiaddr,
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
    convert::Infallible,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

type ListenerUpgrade =
    Pin<Box<(dyn Future<Output = Result<(libp2p::PeerId, StreamMuxerBox), io::Error>> + Send + 'static)>>;
type PeerListener = Pin<Box<dyn Stream<Item = Result<ListenerEvent<ListenerUpgrade, io::Error>, io::Error>> + Send>>;

pub static NUM_LISTENER_EVENT_PROCESSING_ERRORS: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    pub static ref LISTEN_ADDRESSES: Arc<RwLock<Vec<Multiaddr>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Default)]
pub struct ConnectionManager {}

pub struct ConnectionManagerConfig {
    peers: PeerList,
    banned_addrs: BannedAddrList,
    banned_peers: BannedPeerList,
    peer_listener: PeerListener,
    internal_event_sender: InternalEventSender,
}

impl ConnectionManagerConfig {
    pub async fn new(
        local_keys: identity::Keypair,
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

        let listen_address = if let Some(Ok(ListenerEvent::NewAddress(listen_address))) = peer_listener.next().await {
            trace!("listening address = {}", listen_address);
            listen_address
        } else {
            return Err(Error::NotListeningError);
        };

        trace!("Accepting connections on {}.", listen_address);

        LISTEN_ADDRESSES.write().unwrap().push(listen_address);

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
impl<N: Node> Worker<N> for ConnectionManager {
    type Config = ConnectionManagerConfig;
    type Error = Infallible;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let ConnectionManagerConfig {
            peers,
            banned_peers,
            banned_addrs,
            peer_listener,
            internal_event_sender,
            ..
        } = config;

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Listener started.");

            let mut incoming = ShutdownStream::new(shutdown, peer_listener);

            while let Some(Ok(listener_event)) = incoming.next().await {
                //
                if let Some((upgrade, peer_address)) = listener_event.into_upgrade() {
                    // TODO: try again to move this block into its own function (beware: lifetime issues ahead!!!)

                    // Prevent accepting from banned addresses.
                    let peer_address_str = peer_address.to_string();
                    if banned_addrs.contains(&peer_address_str) {
                        trace!("Ignoring peer. Cause: {} is banned.", peer_address_str);
                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }

                    let (peer_id, muxer) = match upgrade.await {
                        Ok(u) => u,
                        Err(_) => {
                            trace!("Ignoring peer. Cause: Handshake failed.");
                            NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                            continue;
                        }
                    };

                    let peer_id = peer_id.into();

                    // Prevent accepting duplicate connections.
                    if let Ok(connected) = peers.is(&peer_id, |_, state| state.is_connected()).await {
                        if connected {
                            trace!("Already connected to {}", peer_id);
                            NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                            continue;
                        }
                    }

                    // Prevent accepting banned peers.
                    if banned_peers.contains(&peer_id) {
                        trace!("Ignoring peer. Cause: {} is banned.", peer_id);
                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }

                    let peer_info = if let Ok(peer_info) = peers.get_info(&peer_id).await {
                        // If we have this peer id in our peerlist (but are not already connected to it),
                        // then we allow the connection.
                        peer_info
                    } else {
                        // We also allow for a certain number of unknown peers.
                        let peer_info = PeerInfo {
                            address: peer_address,
                            alias: peer_id.short().to_string(),
                            relation: PeerRelation::Unknown,
                        };

                        if let Err(e) = peers.accepts(&peer_id, &peer_info).await {
                            trace!("Unknown peer rejected. Cause: {}.", e);
                            NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                            continue;
                        } else {
                            // We also allow for a certain number of unknown peers.
                            info!("Unknown peer '{}' accepted.", peer_info.alias);

                            peer_info
                        }
                    };

                    log_inbound_connection_success(&peer_info);

                    if let Err(e) = super::upgrade_connection(
                        &peer_id,
                        peer_info,
                        muxer,
                        Origin::Inbound,
                        internal_event_sender.clone(),
                    )
                    .await
                    {
                        error!("Error occurred during upgrading the connection. Cause: {}", e);
                        NUM_LISTENER_EVENT_PROCESSING_ERRORS.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                }
            }

            info!("Listener stopped.")
        });

        trace!("Connection Manager started.");

        Ok(Self::default())
    }

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error> {
        info!("Stopping spawned tasks...");
        Ok(())
    }
}
fn log_inbound_connection_success(peer_info: &PeerInfo) {
    info!("Established (inbound) connection with '{}'.", peer_info.alias);
}
