// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::PeerInfo;

use libp2p::{Multiaddr, PeerId};

// NB: We use this type to return ownership back to the caller in case the insertion into the peerlist failed.
pub struct InsertionFailure(pub PeerId, pub PeerInfo, pub Error);

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// A failure due to a peer not being present in the peerlist.
    #[error("Missing peer: {}", .0)]
    PeerMissing(PeerId),
    /// A failure due to attempting to add a peer twice.
    #[error("Already added that peer: {}", .0)]
    PeerAlreadyAdded(PeerId),
    /// A failure due to an address being in the address banlist.
    #[error("Already banned that address: {}", .0)]
    AddressAlreadyBanned(Multiaddr),
    /// A failure due to a peer id being in the peer-id banlist.
    #[error("Already banned that peer: {}", .0)]
    PeerAlreadyBanned(PeerId),
    /// A failure due to attempting to unban an address twice.
    #[error("Already unbanned that address: {}", .0)]
    AddressAlreadyUnbanned(Multiaddr),
    /// A failure due to attempting to unban a peer id twice.
    #[error("Already unbanned that peer: {}", .0)]
    PeerAlreadyUnbanned(PeerId),
    /// A failure due to hitting the maximum number of allowed unknown peers.
    #[error("Tried to add more unknown peers than defined in the config ({}).", .0)]
    UnknownPeerLimitReached(usize),
    /// A failure due to attempting to connect a peer twice.
    #[error("Already connected that peer: {}", .0)]
    PeerAlreadyConnected(PeerId),
    /// A failure due to attempting to disconnect a peer twice.
    #[error("Already disconnected that peer: {}", .0)]
    PeerAlreadyDisconnected(PeerId),
}
