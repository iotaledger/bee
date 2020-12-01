// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peers::PeerRelation;

use libp2p::{Multiaddr, PeerId};

pub type CommandSender = flume::Sender<Command>;
pub type CommandReceiver = flume::Receiver<Command>;

pub fn channel() -> (CommandSender, CommandReceiver) {
    flume::unbounded()
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    AddPeer {
        id: PeerId,
        address: Multiaddr,
        alias: Option<String>,
        relation: PeerRelation,
    },
    RemovePeer {
        id: PeerId,
    },
    ConnectPeer {
        id: PeerId,
    },
    DisconnectPeer {
        id: PeerId,
    },
    DialAddress {
        address: Multiaddr,
    },
    SendMessage {
        message: Vec<u8>,
        to: PeerId,
    },
    BanAddress {
        address: Multiaddr,
    },
    BanPeer {
        id: PeerId,
    },
    UnbanAddress {
        address: Multiaddr,
    },
    UnbanPeer {
        id: PeerId,
    },
    UpdateRelation {
        id: PeerId,
        relation: PeerRelation,
    },
}
