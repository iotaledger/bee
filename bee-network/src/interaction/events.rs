// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conns::Origin,
    peers::{DataSender, PeerInfo},
    Multiaddr, PeerId,
};

use super::commands::Command;

use tokio::sync::mpsc;

pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
pub type EventSender = mpsc::UnboundedSender<Event>;
pub type InternalEventReceiver = mpsc::UnboundedReceiver<InternalEvent>;
pub type InternalEventSender = mpsc::UnboundedSender<InternalEvent>;

pub fn channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    PeerAdded { id: PeerId },
    PeerRemoved { id: PeerId },
    PeerConnected { id: PeerId, address: Multiaddr },
    PeerDisconnected { id: PeerId },
    MessageReceived { message: Vec<u8>, from: PeerId },
    PeerBanned { id: PeerId },
    AddressBanned { address: Multiaddr },
    CommandFailed { command: Command }, // TODO: maybe we should provide the reason as well!
}

#[derive(Debug)]
pub enum InternalEvent {
    ConnectionEstablished {
        peer_id: PeerId,
        peer_info: PeerInfo,
        origin: Origin,
        message_sender: DataSender,
    },
    ConnectionDropped {
        peer_id: PeerId,
    },
    MessageReceived {
        message: Vec<u8>,
        from: PeerId,
    },
    ReconnectScheduled {
        peer_id: PeerId,
    },
}
