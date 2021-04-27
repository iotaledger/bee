// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{Multiaddr, PeerId};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Already added that address: {}", .0)]
    AddressIsAdded(Multiaddr),

    #[error("Already banned that address: {}", .0)]
    AddressIsBanned(Multiaddr),

    #[error("Address is one of the local bind addresses: {}", .0)]
    AddressIsLocal(Multiaddr),

    #[error("Already unbanned that address: {}", .0)]
    AddressIsUnbanned(Multiaddr),

    #[error("Peer matches the local Id: {}", .0)]
    PeerIsLocal(PeerId),

    /// A failure due to a peer not being present in the peerlist.
    #[error("Not present peer: {}", .0)]
    PeerNotPresent(PeerId),

    /// A failure due to attempting to add a peer twice.
    #[error("Already added that peer: {}", .0)]
    PeerIsAdded(PeerId),

    /// A failure due to a peer id being in the peer-id banlist.
    #[error("Already banned that peer: {}", .0)]
    PeerIsBanned(PeerId),

    /// A failure due to attempting to connect a peer twice.
    #[error("Already connected that peer: {}", .0)]
    PeerIsConnected(PeerId),

    /// A failure due to attempting to disconnect a peer twice.
    #[error("Already disconnected that peer: {}", .0)]
    PeerIsDisconnected(PeerId),

    /// A failure due to attempting to unban a peer id twice.
    #[error("Already unbanned that peer: {}", .0)]
    PeerIsUnbanned(PeerId),

    /// A failure due to hitting the maximum number of allowed unknown peers.
    #[error("Tried to add more unknown peers than defined in the config ({}).", .0)]
    ExceedsUnknownPeerLimit(usize),
}
