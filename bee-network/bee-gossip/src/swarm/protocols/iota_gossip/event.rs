// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::io;

use libp2p::{
    swarm::{ConnectionHandlerUpgrErr, NegotiatedSubstream},
    Multiaddr, PeerId,
};

use crate::network::origin::Origin;

/// Gossip events that may occur while establishing the IOTA gossip protocol with a peer.
#[derive(Debug)]
pub enum IotaGossipEvent {
    /// Received IOTA gossip request.
    ReceivedUpgradeRequest { from: PeerId },

    /// Sent IOTA gossip request.
    SentUpgradeRequest { to: PeerId },

    /// The negotiation was successfully completed.
    UpgradeCompleted {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        origin: Origin,
        substream: Box<NegotiatedSubstream>,
    },

    /// An error occured during negotiation.
    UpgradeError {
        peer_id: PeerId,
        error: ConnectionHandlerUpgrErr<io::Error>,
    },
}

/// Gossip handler events that may occur while establishing the IOTA gossip protocol with a peer.
#[derive(Debug)]
pub enum IotaGossipHandlerEvent {
    /// Waiting for an upgrade request when inbound.
    AwaitingUpgradeRequest { from: PeerId },

    /// Received request for IOTA gossip protocol upgrade.
    ReceivedUpgradeRequest { from: PeerId },

    /// Sent request for IOTA gossip protocol upgrade.
    SentUpgradeRequest { to: PeerId },

    /// Successfully upgraded to the IOTA gossip protocol.
    UpgradeCompleted { substream: Box<NegotiatedSubstream> },

    /// An errror occured during the upgrade.
    UpgradeError {
        peer_id: PeerId,
        error: ConnectionHandlerUpgrErr<io::Error>,
    },
}
