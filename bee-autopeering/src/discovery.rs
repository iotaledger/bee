// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{identity::PeerId, peer::Peer};

// From `iotaledger/hive.go`:
// time interval after which the next peer is reverified
const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10;
// time interval after which peers are queried for new peers
const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60;
// maximum number of peers that can be managed
const DEFAULT_MAX_MANAGED: usize = 1000;
// maximum number of peers kept in the replacement list
const DEFAULT_MAX_REPLACEMENTS: usize = 10;

#[derive(Debug)]
pub(crate) enum Event {
    PeerDiscovered { peer: Peer },
    PeerDeleted { peer_id: PeerId },
}

pub(crate) struct DiscoveredPeer {
    peer: Peer,
    // how often that peer has been re-verified
    verified_count: usize,
    // number of returned new peers when queried the last time
    last_new_peers: usize,
}

pub(crate) struct DiscoveryManager {}

impl DiscoveryManager {
    pub(crate) fn new() -> Self {
        Self {}
    }
    pub(crate) async fn run(self) {
        todo!()
    }
}
