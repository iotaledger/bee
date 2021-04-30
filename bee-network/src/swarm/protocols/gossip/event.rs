// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::network::meta::ConnectionInfo;

use libp2p::{swarm::NegotiatedSubstream, Multiaddr, PeerId};

pub struct GossipEvent {
    pub peer_id: PeerId,
    pub peer_multiaddr: Multiaddr,
    pub connection_info: ConnectionInfo,
    pub negotiated_stream: NegotiatedSubstream,
}

#[derive(Default)]
pub struct GossipEventBuilder {
    peer_id: Option<PeerId>,
    peer_addr: Option<Multiaddr>,
    connection_info: Option<ConnectionInfo>,
    negotiated_stream: Option<NegotiatedSubstream>,
}

impl GossipEventBuilder {
    pub fn with_peer_id(mut self, peer_id: PeerId) -> Self {
        self.peer_id.replace(peer_id);
        self
    }

    pub fn with_peer_addr(mut self, peer_addr: Multiaddr) -> Self {
        self.peer_addr.replace(peer_addr);
        self
    }

    pub fn with_conn(mut self, conn: NegotiatedSubstream) -> Self {
        self.negotiated_stream.replace(conn);
        self
    }

    pub fn with_conn_info(mut self, conn_info: ConnectionInfo) -> Self {
        self.connection_info.replace(conn_info);
        self
    }

    pub fn finish(self) -> GossipEvent {
        // Panic:
        // Due to the design it is guaranteed that when this method is called all options have been replaced with valid
        // values. Unwrapping is therefore fine at this point.
        GossipEvent {
            // The following 3 fields are received during `inject_connection_established` which is invoked before
            // `inject_event`.
            peer_id: self.peer_id.unwrap(),
            peer_multiaddr: self.peer_addr.unwrap(),
            negotiated_stream: self.negotiated_stream.unwrap(),

            // This field is received separatedly during `inject_event`.
            connection_info: self.connection_info.unwrap(),
        }
    }
}
