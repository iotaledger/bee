// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// From hive.go:
// * responds to incoming messages
// * sends own requests when needed

use crate::discovery::PING_EXPIRATION;
use crate::store::InMemoryPeerStore;
use crate::{discovery::DiscoveryManager, server::ServerSocket, LocalId};
use crate::{Peer, PeerId};

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) struct DiscoveryProtocol {
    local_id: LocalId,
    socket: ServerSocket,
    version: u32,
    network_id: u32,
    discovery_mngr: DiscoveryManager, // hive.go: actual peer discovery and re-verification
    running: AtomicBool,
}

impl DiscoveryProtocol {
    pub(crate) fn new(
        local_id: LocalId,
        socket: ServerSocket,
        version: u32,
        network_id: u32,
        discovery_mngr: DiscoveryManager,
    ) -> Self {
        Self {
            local_id,
            socket,
            version,
            network_id,
            discovery_mngr,
            running: AtomicBool::new(false),
        }
    }

    pub(crate) async fn start(&self) {
        // TODO
        // start manager
        self.running.swap(true, Ordering::Relaxed);
    }

    pub(crate) async fn stop(&self) {
        // stop manager
        self.running.swap(false, Ordering::Relaxed);
    }
}

// whether the peer has recently done an endpoint proof
pub(crate) fn is_verified(peer_id: &PeerId, addr: IpAddr) -> bool {
    // time.Since(p.loc.Database().LastPong(id, ip)) < PingExpiration
    todo!()
}

// checks whether the given peer has recently sent a Ping;
// if not, we send a Ping to trigger a verification.
pub(crate) fn ensure_verified(peer: Peer) {
    // if that peer has not send a Ping, send a Ping and wait
    // Hive.go: Wait for them to Ping back and process our pong
    // time.Sleep(server.ResponseTimeout)
    todo!()
}

// whether the given peer has recently verified the local peer
pub(crate) fn has_verified(peer_id: &PeerId, addr: IpAddr) -> bool {
    // time.Since(p.loc.Database().LastPing(id, ip)) < PingExpiration
    todo!()
}

// GetMasterPeers returns the list of master peers.
pub(crate) fn get_master_peers() -> Vec<Peer> {
    // simply calls the manager API and forwards the result
    todo!()
}

// GetVerifiedPeer returns the verified peer with the given ID, or nil if no such peer exists.
pub(crate) fn get_verified_peer(peer_id: &PeerId) -> Peer {
    // *iterates all peers returned by `get_verified_peers` of the manager API and compares
    // the peer ids until found
    // * if the peer is not in the manager, it tries to fetch the peer from the db and sends
    // a ping to it in order to verify it
    todo!()
}

// GetVerifiedPeers returns all the currently managed peers that have been verified at least once.
pub(crate) fn get_verified_peers() -> Vec<Peer> {
    // * forwards the call to the manager
    todo!()
}

// HandleMessage responds to incoming peer discovery messages.
pub(crate) fn handle_message(socket: ServerSocket, from_addr: IpAddr, from_id: PeerId, data: Vec<u8>) -> bool {
    // * returns false if `self.running == false`
    // * returns true only if a valid MessageType was found
    // * matches the MessageType byte `data[0]`
    //   Ping:
    //      - creates the protobuf Ping type from `data[1:]`
    //      - calls p.validatePing(from_addr) -> bool, and if okay, calls 'p.handlePing(..)' with all the
    //          passed information
    //   Pong:
    //      - creates the protobuf Pong type from `data[1:]`
    //      - calls p.validatePong(s, fromAddr, from.ID(), m) -> bool, and if okay, calls 'p.handlePong(fromAddr, from, m)
    todo!()
}
