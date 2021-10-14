// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backoff::{Backoff, BackoffBuilder, BackoffMode},
    config::AutopeeringConfig,
    discovery_messages::{Ping, PingFactory},
    identity::{LocalId, PeerId},
    multiaddr::AutopeeringMultiaddr,
    packet::{IncomingPacket, MessageType, OutgoingPacket, Packet, Socket},
    peer::Peer,
};

use tokio::sync::mpsc;

use std::{collections::VecDeque, fmt, net::SocketAddr};

type BootstrapPeer = Peer;

// hive.go: time interval after which the next peer is reverified
const DEFAULT_REVERIFY_INTERVAL_SECS: u64 = 10;
// hive.go: time interval after which peers are queried for new peers
const DEFAULT_QUERY_INTERVAL_SECS: u64 = 60;
// hive.go: maximum number of peers that can be managed
const DEFAULT_MAX_MANAGED: usize = 1000;
// maximum number of peers kept in the replacement list
const DEFAULT_MAX_REPLACEMENTS: usize = 10;
// TODO:
const BACKOFF_INTERVALL_MILLISECS: u64 = 500;
// TODO:
const MAX_RETRIES: usize = 2;
// hive.go: PingExpiration is the time until a peer verification expires (12 hours)
const PING_EXPIRATION: u64 = 12 * 60 * 60;
// hive.go: MaxPeersInResponse is the maximum number of peers returned in DiscoveryResponse.
const MAX_PEERS_IN_RESPONSE: usize = 6;
// hive.go: MaxServices is the maximum number of services a peer can support.

pub(crate) struct DiscoveryConfig {
    pub entry_nodes: Vec<AutopeeringMultiaddr>,
    pub entry_nodes_prefer_ipv6: bool,
    pub run_as_entry_node: bool,
    pub version: u32,
    pub network_id: u32,
    pub source_addr: SocketAddr,
}

impl DiscoveryConfig {
    pub fn new(config: &AutopeeringConfig, version: u32, network_id: u32) -> Self {
        Self {
            entry_nodes: config.entry_nodes.clone(),
            entry_nodes_prefer_ipv6: config.entry_nodes_prefer_ipv6,
            run_as_entry_node: config.run_as_entry_node,
            version,
            network_id,
            source_addr: config.bind_addr,
        }
    }
}

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

impl DiscoveredPeer {
    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer.peer_id()
    }

    pub fn verified_count(&self) -> usize {
        self.verified_count
    }

    pub fn last_new_peers(&self) -> usize {
        self.last_new_peers
    }
}

impl From<Peer> for DiscoveredPeer {
    fn from(peer: Peer) -> Self {
        Self {
            peer,
            verified_count: 0,
            last_new_peers: 0,
        }
    }
}

impl Into<Peer> for DiscoveredPeer {
    fn into(self) -> Peer {
        self.peer
    }
}

impl fmt::Debug for DiscoveredPeer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscoveredPeer")
            .field("peer", &self.peer)
            .field("verified_count", &self.verified_count)
            .field("last_new_peers", &self.last_new_peers)
            .finish()
    }
}

pub(crate) struct DiscoveredPeerlist {
    peers: VecDeque<DiscoveredPeer>,
}

impl DiscoveredPeerlist {
    pub fn new() -> Self {
        Self {
            peers: VecDeque::with_capacity(DEFAULT_MAX_MANAGED),
        }
    }

    pub fn remove(&mut self, peer_id: PeerId) {
        if let Some(index) = self.peers.iter().position(|peer| peer.peer_id() == peer_id) {
            self.remove_at(index)
        }
    }

    pub fn remove_at(&mut self, index: usize) {
        self.peers.remove(index);
    }

    pub fn push_front(&mut self, peer: DiscoveredPeer) {
        self.peers.push_front(peer)
    }

    pub fn push_back(&mut self, peer: DiscoveredPeer) {
        self.peers.push_back(peer)
    }

    pub fn pop_front(&mut self) {
        self.peers.pop_front();
    }

    pub fn pop_back(&mut self) {
        self.peers.pop_back();
    }
}

pub(crate) struct DiscoveryManager {
    // Config.
    config: DiscoveryConfig,
    // The local id to sign outgoing packets.
    local_id: LocalId,
    // Backoff logic.
    backoff: Backoff,
    // Factory to build `Ping` messages.
    ping_factory: PingFactory,
    // Channel halfs for sending/receiving discovery related packets.
    socket: Socket,
}

impl DiscoveryManager {
    pub(crate) fn new(config: DiscoveryConfig, local_id: LocalId, socket: Socket) -> Self {
        let backoff = BackoffBuilder::new(BackoffMode::Exponential(BACKOFF_INTERVALL_MILLISECS, 1.5))
            .with_jitter(0.5)
            .with_max_retries(MAX_RETRIES)
            .finish();

        let ping_factory = PingFactory::new(config.version, config.network_id, config.source_addr);

        Self {
            config,
            local_id,
            backoff,
            ping_factory,
            socket,
        }
    }

    pub(crate) async fn run(self) {
        let DiscoveryManager {
            config,
            local_id,
            backoff,
            ping_factory,
            socket,
        } = self;

        // Send a `Ping` to all entry nodes.
        let bootstrap_peers = config
            .entry_nodes
            .iter()
            .map(|addr| addr.host_socketaddr())
            .collect::<Vec<_>>();

        for target_addr in bootstrap_peers {
            let ping = ping_factory.make(target_addr.ip());
            let msg_bytes = ping.protobuf().expect("error encoding ping");
            let signature = local_id.sign(&msg_bytes);
            let packet = Packet::new(MessageType::Ping, &msg_bytes, &local_id.public_key(), signature);
            socket.send(OutgoingPacket { packet, target_addr });
        }
    }
}
