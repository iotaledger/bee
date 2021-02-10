// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peers::PeerRelation;

use libp2p::{Multiaddr, PeerId};

use tokio::sync::mpsc;

pub type CommandReceiver = mpsc::UnboundedReceiver<Command>;
pub type CommandSender = mpsc::UnboundedSender<Command>;

pub fn command_channel() -> (CommandSender, CommandReceiver) {
    mpsc::unbounded_channel()
}

/// Describes the commands accepted by the networking layer.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Command {
    /// Adds a peer.
    AddPeer {
        /// The peer's id.
        peer_id: PeerId,
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
        peer_id: PeerId,
    },
    /// Connects a peer.
    DialPeer {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Dials an address.
    DialAddress {
        /// The peer's address.
        address: Multiaddr,
    },
    /// Disconnects a peer.
    DisconnectPeer {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Bans a peer.
    BanPeer {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Unbans a peer.
    UnbanPeer {
        /// The peer's id.
        peer_id: PeerId,
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
    /// Upgrades the relation with a peer.
    UpgradeRelation {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Downgrades the relation with a peer.
    DowngradeRelation {
        /// The peer's id.
        peer_id: PeerId,
    },
    /// Discovers new peers.
    DiscoverPeers,
}
