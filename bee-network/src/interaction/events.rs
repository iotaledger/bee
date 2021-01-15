// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conns::Origin,
    peers::{DataSender, PeerInfo},
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
    },
    /// A peer was disconnected.
    PeerDisconnected {
        /// The peer's id.
        id: PeerId,
    },
    /// A message was received.
    MessageReceived {
        /// The received message.
        message: Vec<u8>,
        /// The sender's id.
        from: PeerId,
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
        message_sender: DataSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
    },
    MessageReceived {
        message: Vec<u8>,
        from: PeerId,
    },
    ReconnectScheduled {
        peer_id: PeerId,
    },
}
