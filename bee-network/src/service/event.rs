// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::command::Command;

use crate::{
    network::meta::ConnectionInfo,
    peer::error::Error as PeerError,
    swarm::protocols::gossip::{GossipReceiver, GossipSender},
    types::PeerInfo,
};

use libp2p::{Multiaddr, PeerId};
use tokio::sync::mpsc;

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type InternalEventReceiver = mpsc::UnboundedReceiver<InternalEvent>;
pub type InternalEventSender = mpsc::UnboundedSender<InternalEvent>;

pub fn event_channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

/// Describes the events produced by the networking layer.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// An address was bound.
    AddressBound {
        /// The assigned bind address.
        address: Multiaddr,
    },
    /// The local peer was created.
    LocalCreated {
        /// The created peer id from the Ed25519 keypair.
        peer_id: PeerId,
    },
    /// A peer was added.
    PeerAdded {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's info.
        info: PeerInfo,
    },
    /// A peer was removed.
    PeerRemoved {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// A peer was connected.
    PeerConnected {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's address.
        address: Multiaddr,
        /// The peer's message recv channel.
        gossip_in: GossipReceiver,
        /// The peer's message send channel.
        gossip_out: GossipSender,
    },
    /// A peer was disconnected.
    PeerDisconnected {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// A peer was banned.
    PeerBanned {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// An address was banned.
    AddressBanned {
        /// The peer's address.
        address: Multiaddr,
    },
    /// A command failed.
    CommandFailed {
        /// The command that failed.
        command: Command,
        /// The reason for the failure.
        reason: PeerError,
    },
}

#[derive(Debug)]
pub enum InternalEvent {
    /// An address was bound.
    AddressBound {
        /// The assigned bind address.
        address: Multiaddr,
    },
    ProtocolEstablished {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        conn_info: ConnectionInfo,
        gossip_in: GossipReceiver,
        gossip_out: GossipSender,
    },
    ProtocolDropped {
        peer_id: PeerId,
    },
}
