// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{conns::Error as ConnectionError, Multiaddr, PeerId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to send an event ({}).", .0)]
    EventSendFailure(&'static str),
    #[error("Failed to send an internal event ({}).", .0)]
    InternalEventSendFailure(&'static str),
    // #[error("Failed to send a message to {}", .0)]
    // SendMessageFailure(String),
    #[error("Unlisted peer: {}", .0)]
    UnlistedPeer(PeerId),
    // #[error("Disconnected peer: {}", .0)]
    // DisconnectedPeer(String),
    #[error("Failed to connect to peer. Cause: {:?}", .0)]
    ConnectFailure(ConnectionError),
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
    #[error("Tried to add more unknown peers than allowed ({}).", .0)]
    UnknownPeerLimitReached(usize),
}
