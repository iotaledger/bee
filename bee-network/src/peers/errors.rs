// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p::{Multiaddr, PeerId};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    // #[error("Failed to send an event ({}).", .0)]
    // EventSendFailure(&'static str),
    // #[error("Failed to send an internal event ({}).", .0)]
    // InternalEventSendFailure(&'static str),
    #[error("Unregistered peer: {}", .0)]
    UnregisteredPeer(PeerId),
    // #[error("Failed to connect to peer. Cause: {:?}", .0)]
    // ConnectFailure(ConnError),
    #[error("Already banned that address: {}", .0)]
    AddressAlreadyBanned(Multiaddr),
    #[error("Already banned that peer: {}", .0)]
    PeerAlreadyBanned(PeerId),
    #[error("Already unbanned that address: {}", .0)]
    AddressAlreadyUnbanned(Multiaddr),
    #[error("Already unbanned that peer: {}", .0)]
    PeerAlreadyUnbanned(PeerId),
    #[error("Already added that peer: {}", .0)]
    PeerAlreadyAdded(PeerId),
    #[error("Tried to add more unknown peers than defined in the config ({}).", .0)]
    UnknownPeerLimitReached(usize),
}
