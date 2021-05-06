// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "full")]

use libp2p::{Multiaddr, PeerId};

/// [`PeerList`] errors.
#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// A failure due to an address being added twice.
    #[error("Already added that address: {0}")]
    AddressIsDuplicate(Multiaddr),

    /// A failure due to an address being banned.
    #[error("Already banned that address: {0}")]
    AddressIsBanned(Multiaddr),

    /// A failure due to an address being one of the bind addresses.
    #[error("Address is one of the local bind addresses: {0}")]
    AddressIsLocal(Multiaddr),

    /// A failure due to an address being already unbanned.
    #[error("Already unbanned that address: {0}")]
    AddressIsUnbanned(Multiaddr),

    /// A failure due to a peer id being equal to the local id.
    #[error("Peer matches the local Id: {0}")]
    PeerIsLocal(PeerId),

    /// A failure due to a peer not being present in the peerlist.
    #[error("Not present peer: {0}")]
    PeerNotPresent(PeerId),

    /// A failure due to attempting to add a peer twice.
    #[error("Already added that peer: {0}")]
    PeerIsDuplicate(PeerId),

    /// A failure due to a peer id being banned.
    #[error("Already banned that peer: {0}")]
    PeerIsBanned(PeerId),

    /// A failure due to attempting to connect a peer twice.
    #[error("Already connected that peer: {0}")]
    PeerIsConnected(PeerId),

    /// A failure due to attempting to disconnect a peer twice.
    #[error("Already disconnected that peer: {0}")]
    PeerIsDisconnected(PeerId),

    /// A failure due to attempting to unban a peer id twice.
    #[error("Already unbanned that peer: {0}")]
    PeerIsUnbanned(PeerId),

    /// A failure due to hitting the maximum number of allowed unknown peers.
    #[error("Tried to add more unknown peers than defined in the config ({0}).")]
    ExceedsUnknownPeerLimit(usize),
}
