// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create the transport layer.")]
    CreatingTransportLayerFailed,
    #[error("Binding to {} failed.", .0)]
    BindingFailed(Multiaddr),
    #[error("Tried to dial own peer Id: {}.", .0)]
    DialedOwnPeerId(PeerId),
    #[error("Tried to dial own address: {}.", .0)]
    DialedOwnAddress(Multiaddr),
    #[error("Tried to dial a banned address: {}.", .0)]
    DialedBannedAddress(Multiaddr),
    #[error("Tried to dial a banned peer: {}.", .0)]
    DialedBannedPeer(PeerId),
    #[error("Tried to dial an unregistered peer: {}.", .0)]
    DialedUnregisteredPeer(PeerId),
    // TODO: revisit
    #[error("Tried to dial a peer, that is denied by the peerlist: {}.", .0)]
    DialingPeerDenied(PeerId),
    #[error("Failed dialing address: {}.", .0)]
    DialingFailed(Multiaddr),
    #[error("Already connected to peer: {}.", .0)]
    DuplicateConnection(PeerId),
    #[error("Peer identifies with {}, but we expected: {}", .received, .expected)]
    PeerIdMismatch { expected: PeerId, received: PeerId },
}
