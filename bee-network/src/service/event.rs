// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::command::Command;

use crate::{
    network::origin::Origin,
    peer::{error::Error as PeerError, info::PeerInfo},
    swarm::protocols::iota_gossip::{GossipReceiver, GossipSender},
};

use libp2p::{swarm::NegotiatedSubstream, Multiaddr, PeerId};
use tokio::sync::mpsc;

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type InternalEventReceiver = mpsc::UnboundedReceiver<InternalEvent>;
pub type InternalEventSender = mpsc::UnboundedSender<InternalEvent>;

pub fn event_channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

/// Describes the public events produced by the networking layer.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    /// An address was banned.
    AddressBanned {
        /// The peer's address.
        address: Multiaddr,
    },

    /// An address was unbanned.
    AddressUnbanned {
        /// The peer's address.
        address: Multiaddr,
    },

    /// An address was bound.
    AddressBound {
        /// The assigned bind address.
        address: Multiaddr,
    },

    /// A command failed.
    CommandFailed {
        /// The command that failed.
        command: Command,
        /// The reason for the failure.
        reason: PeerError,
    },

    /// The local peer id was created.
    LocalIdCreated {
        /// The created peer id from the Ed25519 keypair.
        local_id: PeerId,
    },

    /// A peer was added.
    PeerAdded {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's info.
        info: PeerInfo,
    },

    /// A peer was banned.
    PeerBanned {
        /// The peer's id.
        peer_id: PeerId,
    },

    /// A peer was connected.
    PeerConnected {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's info.
        info: PeerInfo,
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

    /// A peer was removed.
    PeerRemoved {
        /// The peer's id.
        peer_id: PeerId,
    },

    /// A peer was unbanned.
    PeerUnbanned {
        /// The peer's id.
        peer_id: PeerId,
    },
}

/// Describes the internal events.
#[derive(Debug)]
pub enum InternalEvent {
    /// An address was bound.
    AddressBound {
        /// The assigned bind address.
        address: Multiaddr,
    },

    /// The gossip protocol has been established with a peer.
    ProtocolEstablished {
        /// The peer's id.
        peer_id: PeerId,
        /// The peer's address.
        peer_addr: Multiaddr,
        /// The associated connection info with that peer.
        origin: Origin,
        /// The negotiated substream the protocol is running on.
        substream: Box<NegotiatedSubstream>,
    },

    /// The gossip protocol has been dropped with a peer.
    ProtocolDropped { peer_id: PeerId },
}

/// Allows the user to receive [`Event`]s published by the network layer.
pub struct NetworkEventReceiver(EventReceiver);

impl NetworkEventReceiver {
    pub(crate) fn new(inner: EventReceiver) -> Self {
        Self(inner)
    }

    /// Waits for an event from the network.
    pub async fn recv(&mut self) -> Option<Event> {
        self.0.recv().await
    }
}

impl From<NetworkEventReceiver> for EventReceiver {
    fn from(rx: NetworkEventReceiver) -> EventReceiver {
        rx.0
    }
}
