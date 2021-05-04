// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::network::origin::Origin;

use libp2p::{
    swarm::{NegotiatedSubstream, ProtocolsHandlerUpgrErr},
    Multiaddr, PeerId,
};

use std::io;

/// Event produced by the [`IotaGossip`].
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
        error: ProtocolsHandlerUpgrErr<io::Error>,
    },
}

/// Event produced by the [`IotaGossipHandler`].
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
        error: ProtocolsHandlerUpgrErr<io::Error>,
    },
}
