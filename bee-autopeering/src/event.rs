// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{Peer, PeerId},
    peering::{manager::Status, neighbor::Distance},
};

use tokio::sync::mpsc;

/// Autopeering related events.
#[derive(Debug)]
pub enum Event {
    /// A new peer has been discovered.
    PeerDiscovered {
        /// The discovered peer.
        peer_id: PeerId,
    },
    /// A peer has been deleted (e.g. due to a failed re-verification).
    PeerDeleted {
        /// The corresponding peer identity.
        peer_id: PeerId,
    },
    /// A SaltUpdated event is triggered, when the private and public salt were updated.
    SaltUpdated {
        public_salt_lifetime: u64,
        private_salt_lifetime: u64,
    },
    /// An OutgoingPeering event is triggered, when a valid response of PeeringRequest has been received.
    OutgoingPeering {
        /// The corresponding peer.
        peer: Peer,
        /// The distance between the local and the remote peer.
        distance: Distance,
    },
    /// An IncomingPeering event is triggered, when a valid PeerRequest has been received.
    IncomingPeering {
        /// The corresponding peer.
        peer: Peer,
        /// The distance between the local and the remote peer.
        distance: Distance,
    },
    /// A Dropped event is triggered, when a neighbor is dropped or when a drop message is received.
    PeeringDropped {
        /// The dropped peer.
        peer_id: PeerId,
    },
}

/// Exposes autopeering related events.
pub type EventRx = mpsc::UnboundedReceiver<Event>;
pub(crate) type EventTx = mpsc::UnboundedSender<Event>;

pub(crate) fn event_chan() -> (EventTx, EventRx) {
    mpsc::unbounded_channel::<Event>()
}
