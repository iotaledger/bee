// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{PeerInfo, PeerState};

use libp2p::{Multiaddr, PeerId};

// NB: We use this type to return ownership back to the caller in case the insertion into the peerlist failed.
pub struct InsertionFailure(pub PeerId, pub PeerInfo, pub PeerState, pub Error);

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    /// A failure due to a peer not being present in the peerlist.
    #[error("Unregistered peer: {}", .0)]
    UnregisteredPeer(PeerId),
    /// A failure due to attempting to register a peer twice.
    #[error("Already registered that peer: {}", .0)]
    PeerAlreadyRegistered(PeerId),
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
}
