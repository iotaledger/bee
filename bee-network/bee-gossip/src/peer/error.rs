// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use libp2p::{Multiaddr, PeerId};

/// [`PeerList`](crate::peer::list::PeerList) errors.
#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// A failure due to an address being added twice.
    #[error("already added that address: {0}")]
    AddressIsDuplicate(Multiaddr),

    /// A failure due to an address being banned.
    #[error("already banned that address: {0}")]
    AddressIsBanned(Multiaddr),

    /// A failure due to an address being one of the bind addresses.
    #[error("address is one of the local bind addresses: {0}")]
    AddressIsLocal(Multiaddr),

    /// A failure due to an address being already unbanned.
    #[error("already unbanned that address: {0}")]
    AddressIsUnbanned(Multiaddr),

    /// A failure due to a peer id being equal to the local id.
    #[error("peer matches the local Id: {0}")]
    PeerIsLocal(PeerId),

    /// A failure due to a peer not being present in the peerlist.
    #[error("not present peer: {0}")]
    PeerNotPresent(PeerId),

    /// A failure due to attempting to add a peer twice.
    #[error("already added that peer: {0}")]
    PeerIsDuplicate(PeerId),

    /// A failure due to a peer id being banned.
    #[error("already banned that peer: {0}")]
    PeerIsBanned(PeerId),

    /// A failure due to attempting to connect a peer twice.
    #[error("already connected that peer: {0}")]
    PeerIsConnected(PeerId),

    /// A failure due to attempting to disconnect a peer twice.
    #[error("already disconnected that peer: {0}")]
    PeerIsDisconnected(PeerId),

    /// A failure due to attempting to unban a peer id twice.
    #[error("already unbanned that peer: {0}")]
    PeerIsUnbanned(PeerId),

    /// A failure due to hitting the maximum number of allowed unknown peers.
    #[error("tried to add more unknown peers than defined in the config ({0})")]
    ExceedsUnknownPeerLimit(usize),

    /// A failure due to hitting the maximum number of allowed discovered peers.
    #[error("tried to add more discovered peers than defined in the config ({0})")]
    ExceedsDiscoveredPeerLimit(usize),
}
