// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, peer::Peer};

use tokio::sync::mpsc;

/// Autopeering related events.
#[derive(Debug)]
pub enum Event {
    /// A new peer has been discovered.
    PeerDiscovered {
        /// The discovered peer.
        peer: Peer,
    },
    /// A peer has been deleted (e.g. due to a failed re-verification).
    PeerDeleted {
        /// The corresponding peer identity.
        peer_id: PeerId,
    },
    /// A SaltUpdated event is triggered, when the private and public salt were updated.
    SaltUpdated,
    /// An OutgoingPeering event is triggered, when a valid response of PeeringRequest has been received.
    OutgoingPeering,
    /// An IncomingPeering event is triggered, when a valid PeerRequest has been received.
    IncomingPeering,
    /// A Dropped event is triggered, when a neighbor is dropped or when a drop message is received.
    Dropped,
}

/// Exposes autopeering related events.
pub type EventRx = mpsc::UnboundedReceiver<Event>;
pub(crate) type EventTx = mpsc::UnboundedSender<Event>;

pub(crate) fn event_chan() -> (EventTx, EventRx) {
    mpsc::unbounded_channel::<Event>()
}
