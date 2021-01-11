// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peers::PeerRelation;

use libp2p::{Multiaddr, PeerId};

use tokio::sync::mpsc;

pub type CommandReceiver = mpsc::UnboundedReceiver<Command>;
pub type CommandSender = mpsc::UnboundedSender<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    mpsc::unbounded_channel()
}

/// Describes the commands accepted by the networking layer.
#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    /// Adds a peer.
    AddPeer {
        /// The peer's id.
        id: PeerId,
        /// The peer's address.
        address: Multiaddr,
        /// The peer's optional alias.
        alias: Option<String>,
        /// The relation with that peer.
        relation: PeerRelation,
    },
    /// Removes a peer.
    RemovePeer {
        /// The peer's id.
        id: PeerId,
    },
    /// Connects a peer.
    ConnectPeer {
        /// The peer's id.
        id: PeerId,
    },
    /// Disconnects a peer.
    DisconnectPeer {
        /// The peer's id.
        id: PeerId,
    },
    /// Dials an address.
    DialAddress {
        /// The peer's address.
        address: Multiaddr,
    },
    /// Sends a message to a peer.
    SendMessage {
        /// The message to be sent.
        message: Vec<u8>,
        /// The receiver's id.
        to: PeerId,
    },
    /// Bans a peer.
    BanPeer {
        /// The peer's id.
        id: PeerId,
    },
    /// Unbans a peer.
    UnbanPeer {
        /// The peer's id.
        id: PeerId,
    },
    /// Bans an address.
    BanAddress {
        /// The peer's address.
        address: Multiaddr,
    },
    /// Unbans an address.
    UnbanAddress {
        /// The peer's address.
        address: Multiaddr,
    },
    /// Updates a relation with a peer.
    UpdateRelation {
        /// The peer's id.
        id: PeerId,

        /// The new relation with that peer.
        relation: PeerRelation,
    },
}
