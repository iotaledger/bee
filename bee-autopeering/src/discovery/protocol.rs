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
pub(crate) fn get_verified_peer<S: PeerStore>(
    peer_id: &PeerId,
    active_peers: &ActivePeersList,
    peerstore: &S,
    command_tx: CommandTx,
) -> Option<Peer> {
    let verified_peers = get_verified_peers(active_peers);

    if verified_peers.iter().any(|pe| pe.peer_id() == peer_id) {
        peerstore.fetch_peer(peer_id)
    } else {
        command_tx
            .send(Command::SendVerificationRequest {
                peer_id: peer_id.clone(),
            })
            .expect("error sending verification request");
        None
    }
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
