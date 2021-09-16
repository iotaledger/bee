// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with events published by the network (layer).

use crate::peer::ConnectedPeer;

/// Represents a network event.
pub enum NetworkEvent {
    /// Fired when a peer has been successfully connected and handshaked.
    PeerConnected(ConnectedPeer),
    /// Fired when a peer worker stops.
    #[cfg(feature = "backstage")]
    PeerWorkerEol,
    /// Fired when a peer worker changes its status.
    #[cfg(feature = "backstage")]
    PeerWorkerReport,
}

#[cfg(feature = "backstage")]
mod backstage {
    use super::NetworkEvent;

    use backstage::core::{ActorResult, EolEvent, ReportEvent, ScopeId, Service};

    impl<T> EolEvent<T> for NetworkEvent {
        fn eol_event(_scope_id: ScopeId, _service: Service, _actor: T, _r: ActorResult<()>) -> Self {
            Self::PeerWorkerEol
        }
    }

    impl<T> ReportEvent<T> for NetworkEvent {
        fn report_event(_scope_id: ScopeId, _service: Service) -> Self {
            Self::PeerWorkerReport
        }
    }
}
