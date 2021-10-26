// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// NOTE: this module is temporary until all bits and pieces are in place. It's a straight forward port of the
// corresponding Go code in `hive.go` with the capabilities provided within this crate. It's not likely to stay
// around for long.

use std::net::SocketAddr;

use crate::{
    command::{Command, CommandTx},
    discovery::manager::VERIFICATION_EXPIRATION_SECS,
    packet::{MessageType, OutgoingPacket},
    peer::Peer,
    peerlist::{ActivePeersList, MasterPeersList},
    peerstore::{self, PeerStore},
    request::RequestManager,
    server::ServerTx,
    service_map::AUTOPEERING_SERVICE_NAME,
    time, PeerId,
};

// Hive.go: whether the peer has recently done an endpoint proof
pub(crate) fn is_verified<S: PeerStore>(peer_id: &PeerId, peerstore: &S) -> bool {
    peerstore.last_verification_response(peer_id).map_or(false, |ts| {
        time::since(ts).map_or(false, |since| since < VERIFICATION_EXPIRATION_SECS)
    })
}

// Hive.go: whether the given peer has recently verified the local peer
pub(crate) fn has_verified<S: PeerStore>(peer_id: &PeerId, peerstore: &S) -> bool {
    peerstore.last_verification_request(peer_id).map_or(false, |ts| {
        time::since(ts).map_or(false, |since| since < VERIFICATION_EXPIRATION_SECS)
    })
}

// Hive.go: checks whether the given peer has recently sent a Ping;
// if not, we send a Ping to trigger a verification.
pub(crate) fn ensure_verified<S: PeerStore>(peer_id: &PeerId, peerstore: &S, command_tx: &CommandTx) {
    if !has_verified(peer_id, peerstore) {
        command_tx.send(Command::SendVerificationRequest {
            peer_id: peer_id.clone(),
        });
    }
}

// Hive.go: returns the list of master peers.
pub(crate) fn get_master_peers<S: PeerStore>(master_peers: &MasterPeersList, peerstore: &S) -> Vec<Peer> {
    let mut peers = Vec::with_capacity(master_peers.len());
    peers.extend(master_peers.iter().filter_map(|peer_id| peerstore.get_peer(peer_id)));
    peers
}

// Hive.go: returns the verified peer with the given ID, or nil if no such peer exists.
pub(crate) fn get_verified_peer<S: PeerStore>(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    peerstore: &S,
    command_tx: CommandTx,
) -> Option<Peer> {
    let verified_peers = get_verified_peers(active_peers);

    if verified_peers.contains(peer_id) {
        peerstore.get_peer(peer_id)
    } else {
        command_tx
            .send(Command::SendVerificationRequest {
                peer_id: peer_id.clone(),
            })
            .expect("error sending verification request");
        None
    }
}

// GetVerifiedPeers returns all the currently managed peers that have been verified at least once.
pub(crate) fn get_verified_peers(active_peers: &ActivePeersList) -> Vec<PeerId> {
    let mut peers = Vec::with_capacity(active_peers.read().len());

    peers.extend(active_peers.read().iter().filter_map(|p| {
        if p.metrics().verified_count() > 0 {
            Some(p.peer_id().clone())
        } else {
            None
        }
    }));

    return peers;
}
