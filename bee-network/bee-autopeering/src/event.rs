// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Events published to the user.

use crate::{
    peer::{Peer, PeerId},
    peering::neighbor::Distance,
};

use tokio::sync::mpsc;

use std::fmt;

/// Autopeering related events.
#[derive(Debug)]
pub enum Event {
    /// A new peer has been discovered.
    PeerDiscovered {
        /// The identity of the discovered peer.
        peer_id: PeerId,
    },
    /// A peer has been deleted (e.g. due to a failed re-verification).
    PeerDeleted {
        /// The identity of the deleted peer.
        peer_id: PeerId,
    },
    /// A SaltUpdated event is triggered, when the private and public salt were updated.
    SaltUpdated {
        /// Lifetime of the public salt.
        public_salt_lifetime: u64,
        /// Lifetime of the private salt.
        private_salt_lifetime: u64,
    },
    /// An OutgoingPeering event is triggered, when a valid response of PeeringRequest has been received.
    OutgoingPeering {
        /// The associated peer.
        peer: Peer,
        /// The distance between the local and the remote peer.
        distance: Distance,
    },
    /// An IncomingPeering event is triggered, when a valid PeerRequest has been received.
    IncomingPeering {
        /// The associated peer.
        peer: Peer,
        /// The distance between the local and the remote peer.
        distance: Distance,
    },
    /// A Dropped event is triggered, when a neighbor is dropped or when a drop message is received.
    PeeringDropped {
        /// The identity of the dropped peer.
        peer_id: PeerId,
    },
}

/// Exposes autopeering related events.
pub type EventRx = mpsc::UnboundedReceiver<Event>;
pub(crate) type EventTx = mpsc::UnboundedSender<Event>;

pub(crate) fn event_chan() -> (EventTx, EventRx) {
    mpsc::unbounded_channel::<Event>()
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Event::*;

        match self {
            PeerDiscovered { peer_id } => write!(f, "Discovered: {}.", peer_id),
            PeerDeleted { peer_id } => write!(f, "Removed offline: {}.", peer_id),
            SaltUpdated {
                public_salt_lifetime,
                private_salt_lifetime,
            } => write!(
                f,
                "Salts updated => outbound: {}/ inbound: {}.",
                public_salt_lifetime, private_salt_lifetime,
            ),
            OutgoingPeering { peer, .. } => write!(f, "Peered: {} (outgoing).", peer.peer_id()),
            IncomingPeering { peer, .. } => write!(f, "Peered: {} (incoming).", peer.peer_id()),
            PeeringDropped { peer_id } => write!(f, "Dropped: {}.", peer_id),
        }
    }
}
