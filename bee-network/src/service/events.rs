// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    host::connections::ConnectionInfo,
    peers::PeerInfo,
    swarm::protocols::gossip::{GossipReceiver, GossipSender},
};

use libp2p::{Multiaddr, PeerId};
use tokio::sync::mpsc;

use super::commands::Command;

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type InternalEventReceiver = mpsc::UnboundedReceiver<InternalEvent>;
pub type InternalEventSender = mpsc::UnboundedSender<InternalEvent>;

pub fn event_channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

/// Describes the events produced by the networking layer.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
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
    /// A new peer has been discovered.
    PeerDiscovered {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's address.
        address: Multiaddr,
    },
    /// A command failed.
    CommandFailed {
        /// The command that failed.
        command: Command,
        /// The reason for the failure.
        reason: Reason,
    },
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Reason {
    /// The peer has already been added.
    #[error("")]
    PeerAlreadyAdded,
    /// The peer has already been removed.
    #[error("")]
    PeerAlreadyRemoved,
    /// The peer is already connected.
    #[error("")]
    PeerAlreadyConnected,
    /// The peer is already disconected.
    #[error("")]
    PeerAlreadyDisconnected,
    /// The reason for the command failure is unknown.
    #[error("Command failed for an unknown reason.")]
    Unknown,
}

#[derive(Debug)]
pub enum InternalEvent {
    ConnectionEstablished {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        conn_info: ConnectionInfo,
        gossip_in: GossipReceiver,
        gossip_out: GossipSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
    },
    ReconnectScheduled {
        peer_id: PeerId,
    },
}
