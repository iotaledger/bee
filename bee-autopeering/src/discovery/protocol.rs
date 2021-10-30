// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// NOTE: this module is temporary until all bits and pieces are in place. It's a straight forward port of the
// corresponding Go code in `hive.go` with the capabilities provided within this crate. It's not likely to stay
// around for long.

use crate::{
    command::{Command, CommandTx},
    discovery::manager::VERIFICATION_EXPIRATION_SECS,
    local::service_map::AUTOPEERING_SERVICE_NAME,
    packet::{MessageType, OutgoingPacket},
    peer::{
        peerlist::{ActivePeerEntry, ActivePeersList, MasterPeersList},
        peerstore::{self, PeerStore},
        Peer,
    },
    request::RequestManager,
    server::ServerTx,
    time, PeerId,
};

use std::net::SocketAddr;

use super::begin_verification_request;

// Hive.go: returns the list of master peers.
pub(crate) fn get_master_peers<S: PeerStore>(master_peers: &MasterPeersList, peerstore: &S) -> Vec<Peer> {
    let mut peers = Vec::with_capacity(master_peers.read().len());
    peers.extend(
        master_peers
            .read()
            .iter()
            .filter_map(|peer_id| peerstore.fetch_peer(peer_id)),
    );
    peers
}

// Hive.go: returns the verified peer with the given ID, or nil if no such peer exists.
pub(crate) async fn get_verified_peer<S: PeerStore>(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    request_mngr: &RequestManager<S>,
    peerstore: &S,
    server_tx: &ServerTx,
) -> Option<Peer> {
    // First check if this peer is in the active/managed list and has a positive verified count.
    if let Some(peer_entry) = active_peers.read().find(peer_id) {
        if peer_entry.metrics().verified_count() > 0 {
            return Some(peer_entry.peer().clone());
        }
    }

    // Enforce re/verification.
    begin_verification_request(peer_id, request_mngr, peerstore, server_tx)
        .await
        .map(|_| {
            // Panic: since the verification request was successful, the peer *must* now be in the active list.
            active_peers
                .read()
                .find(peer_id)
                .expect("inconsistent peer list")
                .peer()
                .clone()
        })
}

// Hive.go: returns all the currently managed peers that have been verified at least once.
pub(crate) fn get_verified_peers(active_peers: &ActivePeersList) -> Vec<ActivePeerEntry> {
    let mut peers = Vec::with_capacity(active_peers.read().len());

    peers.extend(active_peers.read().iter().filter_map(|p| {
        if p.metrics().verified_count() > 0 {
            Some(p.clone())
        } else {
            None
        }
    }));

    return peers;
}
