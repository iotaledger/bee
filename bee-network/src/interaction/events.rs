// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conns::Origin,
    peers::{MessageReceiver, MessageSender, PeerInfo},
    Multiaddr, PeerId,
};

use super::commands::Command;

use tokio::sync::mpsc;

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type InternalEventReceiver = mpsc::UnboundedReceiver<InternalEvent>;
pub type InternalEventSender = mpsc::UnboundedSender<InternalEvent>;

pub fn channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

/// Describes the events produced by the networking layer.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// A peer was added.
    PeerAdded {
        /// The peer's id.
        id: PeerId,
        /// The peer's info.
        info: PeerInfo,
    },
    /// A peer was removed.
    PeerRemoved {
        /// The peer's id.
        id: PeerId,
    },
    /// A peer was connected.
    PeerConnected {
        /// The peer's id.
        id: PeerId,
        /// The peer's address.
        address: Multiaddr,
        /// The peer's message recv channel.
        gossip_in: MessageReceiver,
        /// The peer's message send channel.
        gossip_out: MessageSender,
    },
    /// A peer was disconnected.
    PeerDisconnected {
        /// The peer's id.
        id: PeerId,
    },
    /// A peer was banned.
    PeerBanned {
        /// The peer's id.
        id: PeerId,
    },
    /// An address was banned.
    AddressBanned {
        /// The peer's address.
        address: Multiaddr,
    },
    /// A command failed.
    CommandFailed {
        /// The failed command.
        command: Command,
    }, // TODO: maybe we should provide the reason as well!
}

#[derive(Debug)]
pub enum InternalEvent {
    ConnectionEstablished {
        peer_id: PeerId,
        peer_info: PeerInfo,
        origin: Origin,
        gossip_in: MessageReceiver,
        gossip_out: MessageSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
    },
    ReconnectScheduled {
        peer_id: PeerId,
    },
}
