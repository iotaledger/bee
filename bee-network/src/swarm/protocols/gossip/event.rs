// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::network::meta::ConnectionInfo;

use libp2p::{swarm::NegotiatedSubstream, Multiaddr, PeerId};

pub struct GossipEvent {
    pub peer_id: PeerId,
    pub peer_addr: Multiaddr,
    pub conn: NegotiatedSubstream,
    pub conn_info: ConnectionInfo,
}

#[derive(Default)]
pub struct GossipEventBuilder {
    peer_id: Option<PeerId>,
    peer_addr: Option<Multiaddr>,
    conn: Option<NegotiatedSubstream>,
    conn_info: Option<ConnectionInfo>,
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
        self.conn.replace(conn);
        self
    }

    pub fn with_conn_info(mut self, conn_info: ConnectionInfo) -> Self {
        self.conn_info.replace(conn_info);
        self
    }

    pub fn finish(self) -> GossipEvent {
        // Panic:
        // Due to the design it is guaranteed that when this method is called all options have been replaced with valid
        // values. Unwrapping is therefore fine at this point.
        GossipEvent {
            peer_id: self.peer_id.unwrap(),
            peer_addr: self.peer_addr.unwrap(),
            conn: self.conn.unwrap(),
            conn_info: self.conn_info.unwrap(),
        }
    }
}
