// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::{
    gossip::{
        behaviour::GossipEvent,
        layer::{GossipLayer, GossipLayerEvent},
    },
    init::BootError,
    peer::{peer_id::PeerId, peer_state_map::PeerStateMap},
    task::ShutdownRx,
};

use futures::{channel::oneshot, StreamExt};
use libp2p::{
    identify::IdentifyEvent,
    ping::PingEvent,
    swarm::{DialError, NegotiatedSubstream, SwarmEvent},
    Multiaddr,
};
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum DialingError {
    #[error("Dialing address {0} failed. Cause: {1:?}")]
    Failed(PeerId, Multiaddr, DialError),
}

pub(crate) type GossipServerEventTx = mpsc::UnboundedSender<GossipServerEvent>;
pub(crate) type GossipServerEventRx = mpsc::UnboundedReceiver<GossipServerEvent>;
pub(crate) type GossipServerCommandTx = mpsc::UnboundedSender<GossipServerCommand>;
pub(crate) type GossipServerCommandRx = mpsc::UnboundedReceiver<GossipServerCommand>;

pub(crate) use tokio::sync::mpsc::unbounded_channel as chan;

/// Commands accepted by the gossip server.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum GossipServerCommand {
    Dial { peer_id: PeerId, peer_addr: Multiaddr },
    Hangup { peer_id: PeerId },
}

/// Describes the public events produced by the networking layer.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum GossipServerEvent {
    /// The gossip protocol with a peer was established.
    GossipProtocolEstablished {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's address.
        peer_addr: Multiaddr,
        /// The peer's negotiated gossip substream.
        substream: NegotiatedSubstream,
    },
    /// The gossip protocol with a peer was terminated.
    GossipProtocolTerminated {
        /// The peer's id.
        peer_id: PeerId,
    },
}

pub struct GossipServerConfig {
    pub(crate) bind_addr: Multiaddr,
    pub(crate) gossip_layer: GossipLayer,
    pub(crate) peer_state_map: PeerStateMap,
    pub(crate) server_event_tx: GossipServerEventTx,
    pub(crate) server_command_rx: GossipServerCommandRx,
}

pub mod workers {
    use super::*;
    use crate::manager::workers::GossipManager;
    use async_trait::async_trait;
    use bee_runtime::{node::Node, worker::Worker};
    use std::{any::TypeId, convert::Infallible};

    /// A node worker, that deals with accepting and initiating connections with remote peers.
    ///
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct GossipServer {}

    #[async_trait]
    impl<N: Node> Worker<N> for GossipServer {
        type Config = GossipServerConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            vec![TypeId::of::<GossipManager>()].leak()
        }

        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            node.spawn::<Self, _, _>(|shutdown| async move {
                // Start gossip server event loop that listens for events from the network.
                //
                // Panic:
                // The gossip server event loop must not fail.
                gossip_server_command_event_loop(config, shutdown)
                    .await
                    .expect("gossip server event loop");

                log::debug!("Gossip server stopped.");
            });

            log::debug!("Gossip server started.");

            Ok(Self::default())
        }
    }
}

pub(crate) struct GossipServer {
    pub shutdown: oneshot::Receiver<()>,
}

impl GossipServer {
    pub fn new(shutdown: oneshot::Receiver<()>) -> Self {
        Self { shutdown }
    }

    pub async fn start(self, config: GossipServerConfig) {
        let GossipServer { shutdown } = self;

        tokio::spawn(async move {
            gossip_server_command_event_loop(config, shutdown)
                .await
                .expect("gossip server event loop");

            log::debug!("Gossip server stopped.");
        });

        log::debug!("Gossip server started.");
    }
}

async fn gossip_server_command_event_loop(
    config: GossipServerConfig,
    mut shutdown_rx: ShutdownRx,
) -> Result<(), Box<dyn std::error::Error>> {
    let GossipServerConfig {
        bind_addr,
        mut gossip_layer,
        peer_state_map,
        server_event_tx,
        mut server_command_rx,
    } = config;

    log::debug!("Trying to bind gossip server to: {}", bind_addr);

    let _id = gossip_layer
        .listen_on(bind_addr)
        .map_err(|_| BootError::BindGossipServer)?;

    log::debug!("Gossip server command/event loop running.");

    loop {
        tokio::select! {
            // Listen for the shutdown signal.
            _ = &mut shutdown_rx => break,
            // Listen for commands from the manager.
            command = (&mut server_command_rx).recv() => {
                // Panic:
                // The channel must not be empty.
                let command = command.expect("empty command channel");
                handle_server_command(command, &mut gossip_layer, &peer_state_map).await;
            },
            // Listen for events.
            event = gossip_layer.select_next_some() => {
                handle_gossip_layer_event(event, &peer_state_map, &server_event_tx).await;
            }
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// Command handlers
///////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn handle_server_command(
    command: GossipServerCommand,
    gossip_layer: &mut GossipLayer,
    peer_state_map: &PeerStateMap,
) {
    use GossipServerCommand::*;
    match command {
        Dial { peer_id, peer_addr } => {
            if peer_state_map.accept_outbound_peer(&peer_id) {
                if let Err(e) = dial(peer_id, peer_addr.clone(), gossip_layer).await {
                    log::debug!("Dialing address {} failed. Cause: {}", peer_addr, e);
                }
            } else {
                log::warn!("Dialing peer {peer_id} at address {peer_addr} denied.");
            }
        }
        Hangup { peer_id } => {
            if hangup(peer_id, gossip_layer).await.is_err() {
                log::debug!("Hanging up failed for {}.", peer_id);
            }
        }
    }
}

async fn dial(peer_id: PeerId, peer_addr: Multiaddr, gossip_layer: &mut GossipLayer) -> Result<(), DialingError> {
    log::debug!("Dialing {peer_id} at address: {peer_addr}.");

    gossip_layer
        .dial(peer_addr.clone())
        .map_err(|e| DialingError::Failed(peer_id, peer_addr, e))?;

    Ok(())
}

async fn hangup(peer_id: PeerId, gossip_layer: &mut GossipLayer) -> Result<(), ()> {
    // Do we need to go through the `ProtocolsHandler`? See docs for this method.
    gossip_layer.disconnect_peer_id(*peer_id)
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// Event handlers
///////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn handle_gossip_layer_event(
    event: SwarmEvent<GossipLayerEvent, impl std::error::Error>,
    peer_state_map: &PeerStateMap,
    server_event_tx: &GossipServerEventTx,
) {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            log::trace!("Transport event: new listen address {address}.");

            // Panic:
            // Inserting a listen address must not fail.
            peer_state_map.add_local_address(address).expect("add local address");
        }
        SwarmEvent::ConnectionEstablished {
            peer_id,
            endpoint,
            num_established,
            ..
        } => {
            let peer_id: PeerId = peer_id.into();
            log::trace!(
                "Transport event: connection established with {peer_id} as {endpoint:?}. #{num_established} total."
            );
        }
        SwarmEvent::ConnectionClosed {
            peer_id,
            num_established,
            cause,
            ..
        } => {
            let peer_id: PeerId = peer_id.into();
            log::trace!(
                "Transport event: connection closed with {peer_id}. Cause: {cause:?}. #{num_established} remaining."
            );
        }
        SwarmEvent::ListenerError { error, .. } => {
            log::trace!("Transport event: listener error {error}.");
        }
        SwarmEvent::Dialing(peer_id) => {
            let peer_id: PeerId = peer_id.into();
            log::trace!("Transport event: dialing {peer_id}.");
        }
        SwarmEvent::IncomingConnection {
            local_addr,
            send_back_addr,
            ..
        } => {
            log::trace!("Transport event: being dialed from {send_back_addr} at {local_addr}.");
        }
        SwarmEvent::Behaviour(event) => {
            log::trace!("Protocol event...");

            use GossipLayerEvent::*;
            match event {
                Identify(event) => handle_identify_event(event, peer_state_map, server_event_tx),
                Ping(event) => handle_ping_event(event, peer_state_map, server_event_tx),
                Gossip(event) => handle_gossip_event(event, peer_state_map, server_event_tx),
            }
        }
        _ => {}
    }
}

fn handle_identify_event(event: IdentifyEvent, peer_state_map: &PeerStateMap, _: &GossipServerEventTx) {
    use IdentifyEvent::*;
    match event {
        Received { peer_id, info } => {
            let peer_id: PeerId = peer_id.into();

            log::trace!("Received Identify request from {peer_id}. Peer infos: {info:?}.");

            if info.agent_version.contains("hornet") {
                log::trace!("Peer claims to be a Hornet node.");
            } else if info.agent_version.contains("bee") {
                log::trace!("Peer claims to be a Bee node.");
            } else {
                log::trace!("Peer seems to be an unknown node implementation.")
            }

            peer_state_map.update_last_identify(peer_id);
        }
        Sent { peer_id } => {
            let peer_id: PeerId = peer_id.into();

            log::trace!("Sent Identify request to {peer_id}.");
        }
        Pushed { peer_id } => {
            let peer_id: PeerId = peer_id.into();
            log::trace!("Pushed Identify request to {peer_id}.");
        }
        Error { peer_id, error } => {
            let peer_id: PeerId = peer_id.into();

            log::trace!("Identify error with {peer_id}: {error:?}.");

            // TODO: close the connection
        }
    }
}

fn handle_ping_event(event: PingEvent, peer_state_map: &PeerStateMap, _: &GossipServerEventTx) {
    log::trace!("Ping: {event:?}");

    peer_state_map.update_last_ping(event.peer.into());
}

fn handle_gossip_event(event: GossipEvent, _: &PeerStateMap, server_event_tx: &GossipServerEventTx) {
    use GossipEvent::*;
    match event {
        Established {
            peer_id,
            peer_addr,
            substream,
        } => {
            let peer_id: PeerId = peer_id.into();
            log::debug!("Gossip protocol established with {peer_id}");

            server_event_tx
                .send(GossipServerEvent::GossipProtocolEstablished {
                    peer_id,
                    peer_addr,
                    substream,
                })
                .expect("send server event");
        }
        NegotiationError { peer_id, error: _ } => {
            let peer_id: PeerId = peer_id.into();

            log::debug!("Protocol negotiation error for {}", peer_id);
        }
        Terminated { peer_id } => {
            let peer_id: PeerId = peer_id.into();

            log::debug!("Gossip protocol terminated with: {}", peer_id);

            server_event_tx
                .send(GossipServerEvent::GossipProtocolTerminated { peer_id })
                .expect("send server event");
        }
    }
}
