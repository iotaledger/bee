// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use crate::{
    peer::{
        peer_data::{PeerInfo, PeerRelation, PeerType},
        peer_id::PeerId,
        peer_state_map::{InboundPeerAcceptance, PeerStateMap},
    },
    server::{GossipServerCommand, GossipServerCommandTx, GossipServerEvent, GossipServerEventRx},
    task::{self, ShutdownRx},
};

use bee_runtime::shutdown_stream::ShutdownStream;

use futures::{
    io::{BufReader, BufWriter, ReadHalf, WriteHalf},
    AsyncReadExt, AsyncWriteExt, StreamExt,
};
use libp2p::{swarm::NegotiatedSubstream, Multiaddr};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::io;

/// Allows sending [`GossipManagerCommand`]s to the gossip manager.
pub type GossipManagerCommandTx = mpsc::UnboundedSender<GossipManagerCommand>;

/// Allows receiving [`GossipManagerEvent`]s from the gossip manager.
pub type GossipManagerEventRx = mpsc::UnboundedReceiver<GossipManagerEvent>;

pub(crate) type GossipManagerCommandRx = mpsc::UnboundedReceiver<GossipManagerCommand>;
pub(crate) type GossipManagerEventTx = mpsc::UnboundedSender<GossipManagerEvent>;

pub(crate) use tokio::sync::mpsc::unbounded_channel as chan;

/// The buffered writer end of a gossip connection with a peer.
#[derive(Debug)]
pub struct GossipTx(BufWriter<WriteHalf<NegotiatedSubstream>>);

impl GossipTx {
    fn new(buffered_writer: BufWriter<WriteHalf<NegotiatedSubstream>>) -> Self {
        Self(buffered_writer)
    }

    /// Sends a message.
    pub async fn send(&mut self, msg: &[u8]) -> Result<(), io::Error> {
        self.0.write_all(msg).await
        // flush?
    }
}

/// The buffered reader end of a gossip connection with a peer.
#[derive(Debug)]
pub struct GossipRx(BufReader<ReadHalf<NegotiatedSubstream>>, Box<[u8; 8 * 1024]>);

impl GossipRx {
    fn new(buffered_reader: BufReader<ReadHalf<NegotiatedSubstream>>) -> Self {
        Self(buffered_reader, Box::new([0u8; 8 * 1024]))
    }

    /// Receives a message.
    pub async fn recv(&mut self) -> Result<Vec<u8>, io::Error> {
        let n = self.0.read(&mut *self.1).await?;
        let mut msg = vec![0u8; n];
        msg.copy_from_slice(&self.1[..n]);
        Ok(msg)
    }
}

/// Events produced by the gossip manager.
#[derive(Debug)]
pub enum GossipManagerEvent {
    /// A peer was connected.
    PeerConnected {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's info.
        peer_info: PeerInfo,
        /// The peer's gossip stream writer.
        writer: GossipTx,
        /// The peer's gossip stream reader.
        reader: GossipRx,
    },
    /// A peer was disconnected.
    PeerDisconnected {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// A peer doesn't answer our calls.
    PeerUnreachable {
        /// The peer's id.
        peer_id: PeerId,
        /// The relation we have with that peer.
        peer_relation: PeerRelation,
    },
}

/// Commands accepted by the gossip manager.
#[derive(Debug)]
#[non_exhaustive]
pub enum GossipManagerCommand {
    /// Adds a new peer.
    ///
    /// Note:
    /// This implies connecting to that peer.
    AddPeer {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's address.
        peer_addr: Multiaddr,
        /// The peer's optional alias.
        peer_alias: Option<String>,
        /// The peer's type.
        peer_type: PeerType,
    },
    /// Removes a known peer.
    ///
    /// Note:
    /// This implies disconnecting from that peer.
    RemovePeer {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Connects to a known peer.
    ConnectPeer {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Disconnects from a peer.
    DisconnectPeer {
        /// The peer's id.
        peer_id: PeerId,
    },
}

pub struct GossipManagerConfig {
    pub(crate) senders: Senders,
    pub(crate) receivers: Receivers,
    pub(crate) peer_state_map: PeerStateMap,
}

#[derive(Clone)]
pub(crate) struct Senders {
    pub(crate) manager_event_tx: GossipManagerEventTx,
    pub(crate) server_command_tx: GossipServerCommandTx,
}

pub(crate) struct Receivers {
    pub(crate) manager_command_rx: GossipManagerCommandRx,
    pub(crate) server_event_rx: GossipServerEventRx,
}

const BUFFER_SIZE: usize = 32 * 1024;

pub(crate) struct GossipManager {
    shutdown_rx: ShutdownRx,
}

impl GossipManager {
    pub fn new(shutdown_rx: ShutdownRx) -> Self {
        Self { shutdown_rx }
    }

    pub async fn start(self, config: GossipManagerConfig) {
        let GossipManager { shutdown_rx } = self;
        let GossipManagerConfig {
            senders,
            receivers,
            peer_state_map,
        } = config;

        let Receivers {
            manager_command_rx,
            server_event_rx,
        } = receivers;

        let (command_loop_shutdown_tx, command_loop_shutdown_rx) = task::shutdown_chan();
        let (event_loop_shutdown_tx, event_loop_shutdown_rx) = task::shutdown_chan();

        tokio::spawn(async move {
            // Panic:
            // Awaiting the shutdown signal must not fail.
            shutdown_rx.await.expect("await shutdown signal");

            // Panic:
            // Sending shutdown signals must not fail.
            command_loop_shutdown_tx
                .send(())
                .expect("send command loop shutdown signal");
            event_loop_shutdown_tx
                .send(())
                .expect("send event loop shutdown signal");
        });

        // Start gossip manager command loop that is processing incoming commands from the user.
        tokio::spawn(gossip_manager_command_loop(
            command_loop_shutdown_rx,
            manager_command_rx,
            senders.clone(),
            peer_state_map.clone(),
        ));

        // Start gossip manager event loop that is processing incoming events from the gossip server.
        tokio::spawn(gossip_manager_event_loop(
            event_loop_shutdown_rx,
            server_event_rx,
            senders,
            peer_state_map,
        ));

        log::debug!("Gossip manager started.");
    }
}

pub mod workers {
    use super::*;
    use async_trait::async_trait;
    use bee_runtime::{node::Node, worker::Worker};
    use std::{any::TypeId, convert::Infallible};

    /// A node worker, that deals with processing user commands, and publishing events.
    ///
    /// NOTE: This type is only exported to be used as a worker dependency.
    #[derive(Default)]
    pub struct GossipManager {}

    #[async_trait]
    impl<N: Node> Worker<N> for GossipManager {
        type Config = GossipManagerConfig;
        type Error = Infallible;

        fn dependencies() -> &'static [TypeId] {
            &[]
        }

        async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
            let GossipManagerConfig {
                senders,
                receivers,
                peer_state_map,
            } = config;

            let Receivers {
                manager_command_rx,
                server_event_rx,
            } = receivers;

            // Start gossip manager command loop that is processing incoming commands from the user.
            node.spawn::<Self, _, _>(|shutdown| {
                gossip_manager_command_loop(shutdown, manager_command_rx, senders.clone(), peer_state_map.clone())
            });

            // Start gossip manager event loop that is processing incoming events from the gossip server.
            node.spawn::<Self, _, _>(|shutdown| {
                gossip_manager_event_loop(shutdown, server_event_rx, senders.clone(), peer_state_map.clone())
            });

            log::debug!("Gossip manager started.");

            Ok(Self::default())
        }
    }
}

async fn gossip_manager_command_loop(
    shutdown_rx: ShutdownRx,
    manager_command_rx: GossipManagerCommandRx,
    senders: Senders,
    peer_state_map: PeerStateMap,
) {
    log::debug!("Gossip manager command loop running.");

    let mut commands = ShutdownStream::new(shutdown_rx, UnboundedReceiverStream::new(manager_command_rx));

    while let Some(command) = commands.next().await {
        if handle_command(command, &senders, &peer_state_map).await.is_err() {
            log::error!("Error handling manager command.");
            continue;
        }
    }

    log::debug!("Gossip manager command loop stopped.");
}

async fn gossip_manager_event_loop(
    shutdown_rx: ShutdownRx,
    server_event_rx: GossipServerEventRx,
    senders: Senders,
    peer_state_map: PeerStateMap,
) {
    log::debug!("Gossip manager event loop running.");

    let mut server_events = ShutdownStream::new(shutdown_rx, UnboundedReceiverStream::new(server_event_rx));

    while let Some(server_event) = server_events.next().await {
        if handle_server_event(server_event, &senders, &peer_state_map)
            .await
            .is_err()
        {
            log::error!("Error handling server event.");
            continue;
        }
    }

    log::debug!("Gossip manager event loop stopped.");
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// Command handlers
///////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn handle_command(
    command: GossipManagerCommand,
    senders: &Senders,
    peer_state_map: &PeerStateMap,
) -> Result<(), ()> {
    use GossipManagerCommand::*;

    log::debug!("Handling command: {command:?}.");

    match command {
        AddPeer {
            peer_id,
            peer_addr,
            peer_alias,
            peer_type,
        } => {
            // If no alias is given we default to the `Display` representation of the peer id.
            let peer_alias = peer_alias.unwrap_or_else(|| peer_id.to_string());

            handle_add_peer_command(peer_id, peer_addr, peer_alias, peer_type, peer_state_map, senders).await?;
        }
        RemovePeer { peer_id } => {
            handle_remove_peer_command(peer_id, peer_state_map, senders).await?;
        }

        ConnectPeer { peer_id } => {
            handle_connect_peer_command(peer_id, peer_state_map, senders).await?;
        }

        DisconnectPeer { peer_id } => {
            handle_disconnect_peer_command(peer_id, peer_state_map, senders).await?;
        }
    }

    Ok(())
}

async fn handle_add_peer_command(
    peer_id: PeerId,
    peer_addr: Multiaddr,
    peer_alias: String,
    peer_type: PeerType,
    peer_state_map: &PeerStateMap,
    senders: &Senders,
) -> Result<(), ()> {
    let peer_info = PeerInfo {
        address: peer_addr.clone(),
        alias: peer_alias,
        relation: peer_type.into(),
    };

    if peer_state_map.add_remote_peer(peer_id, peer_info).is_ok() {
        log::debug!("Peer {peer_id} added.");

        log::trace!("Sending `Dial` command to server for peer {peer_id}.");

        // Immediatedly try to dial that peer.
        // Panic:
        // Sending commands must not fail.
        senders
            .server_command_tx
            .send(GossipServerCommand::Dial { peer_id, peer_addr })
            .expect("send server command");

        Ok(())
    } else {
        Err(())
    }
}

async fn handle_remove_peer_command(
    peer_id: PeerId,
    peer_state_map: &PeerStateMap,
    senders: &Senders,
) -> Result<(), ()> {
    handle_disconnect_peer_command(peer_id, peer_state_map, senders).await?;

    if peer_state_map.remove_peer(&peer_id).is_ok() {
        log::debug!("Peer {peer_id} removed.");

        Ok(())
    } else {
        Err(())
    }
}

async fn handle_connect_peer_command(
    peer_id: PeerId,
    peer_state_map: &PeerStateMap,
    senders: &Senders,
) -> Result<(), ()> {
    if let Some(peer_info) = peer_state_map.get_info_conditionally(&peer_id, |v| v.peer_state.is_disconnected()) {
        log::trace!("Sending `Dial` command to server for peer {peer_id}.");

        // Panic:
        // Sending a command to the server must not fail.
        senders
            .server_command_tx
            .send(GossipServerCommand::Dial {
                peer_id,
                peer_addr: peer_info.address,
            })
            .expect("send server command");

        Ok(())
    } else {
        Err(())
    }
}

async fn handle_disconnect_peer_command(
    peer_id: PeerId,
    peer_state_map: &PeerStateMap,
    senders: &Senders,
) -> Result<(), ()> {
    if peer_state_map.peer_satisfies_condition(&peer_id, |v| v.peer_state.is_connected()) {
        log::trace!("Sending `Hangup` command to server for peer {peer_id}.");

        // Panic:
        // Sending a command to the server must not fail.
        senders
            .server_command_tx
            .send(GossipServerCommand::Hangup { peer_id })
            .expect("send server command");

        Ok(())
    } else {
        Err(())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////
// Event handlers
///////////////////////////////////////////////////////////////////////////////////////////////////////////

async fn handle_server_event(
    server_event: GossipServerEvent,
    senders: &Senders,
    peer_state_map: &PeerStateMap,
) -> Result<(), ()> {
    use GossipServerEvent::*;

    match server_event {
        GossipProtocolEstablished {
            peer_id,
            peer_addr,
            substream,
        } => {
            let peer_acceptance = peer_state_map.accept_inbound_peer(&peer_id, &peer_addr);
            if peer_acceptance.is_accepted() {
                // If the peer doesn't exist yet - but is accepted as an "unknown" peer, we insert it now.
                if matches!(peer_acceptance, InboundPeerAcceptance::AcceptUnknown) {
                    let peer_info = PeerInfo {
                        address: peer_addr,
                        alias: peer_id.to_string(),
                        relation: PeerRelation::Unknown,
                    };

                    // Panic:
                    // The peer is unknown, so the insert cannot fail.
                    peer_state_map
                        .add_remote_peer(peer_id, peer_info)
                        .expect("add remote peer");
                }

                // Panic:
                // We made sure that the peer exists.
                peer_state_map
                    .update_peer_state(&peer_id, |s| s.set_connected())
                    .expect("update state");

                // Panic:
                // We made sure that the peer exists.
                let peer_info = peer_state_map.get_info(&peer_id).expect("get info");

                // Split the stream into separate reader and writer halves.
                let (reader, writer) = substream.split();

                // Buffer reader/writer to reduce the amount of syscalls.
                let writer = GossipTx::new(BufWriter::with_capacity(BUFFER_SIZE, writer));
                let reader = GossipRx::new(BufReader::with_capacity(BUFFER_SIZE, reader));

                // Publish that data to the user.
                // Panic:
                // Sending must not fail.
                senders
                    .manager_event_tx
                    .send(GossipManagerEvent::PeerConnected {
                        peer_id,
                        peer_info,
                        writer,
                        reader,
                    })
                    .expect("send manager event")
            }
        }
        GossipProtocolTerminated { peer_id } => {
            if let Err(_e) = peer_state_map.update_peer_state(&peer_id, |state| state.set_disconnected()) {
                log::trace!("Failed to change state to disconnected for peer {peer_id}");
            }

            // Only remove unknown peers.
            // Note: known and discovered peers can only be removed via an explicit command.
            if peer_state_map.remove_peer_conditionally(&peer_id, |v| v.peer_info.relation.is_unknown()) {
                log::trace!("Removed unknown peer {peer_id}");
            }

            senders
                .manager_event_tx
                .send(GossipManagerEvent::PeerDisconnected { peer_id })
                .expect("send manager event");

            // TODO: spawn a reconnector task with exponential backoff until final attempt, then we remove Discovered,
            // then we signal that the peer is probably dead.
        }
    }

    Ok(())
}
