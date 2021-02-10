// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Tried to dial a banned address: '{}'.", .0)]
    DialedBannedAddress(Multiaddr),

    #[error("Tried to dial banned peer: '{}'.", .0)]
    DialedBannedPeer(PeerId),

    #[error("Tried to dial oneself: '{}'.", .0)]
    DialedSelf(String),

    #[error("Tried to dial own listen address: '{}'.", .0)]
    DialedOwnAddress(Multiaddr),

    #[error("Tried to dial an unregistered peer: '{}'.", .0)]
    DialedUnregisteredPeer(String),

    // #[error("Tried to dial a peer, that was rejected from the peerlist: '{}'.", .0)]
    // DialedRejectedPeer(String),
    #[error("Failed dialing address: '{}'.", .0)]
    DialingFailed(Multiaddr),

    #[error("Already connected to peer: '{}'.", .0)]
    DuplicateConnection(String),

    #[error("Peer identifies with '{}', but we expected: '{}'", .received, .expected)]
    PeerIdMismatch { expected: String, received: String },

    #[error("Failed to write a message to a stream.")]
    MessageSendError,

    #[error("Failed to read a message from a stream.")]
    MessageRecvError,

    #[error("The remote peer '{}' stopped the stream (EOF).", .0)]
    StreamClosedByRemote(PeerId),
}
