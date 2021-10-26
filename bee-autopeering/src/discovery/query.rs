// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    command::{Command, CommandTx},
    peerlist::ActivePeersList,
    task::Repeat,
};

pub(crate) fn roundrobin_verification() -> Repeat<(ActivePeersList, CommandTx)> {
    Box::new(|(active_peers, command_tx)| {
        if let Some(peer_entry) = active_peers.read().newest() {
            let peer_id = peer_entry.peer_id().clone();

            command_tx
                .send(Command::SendVerificationRequest { peer_id })
                .expect("error sending command");
        }

        active_peers.write().rotate_forwards();
    })
}

pub(crate) fn oldest_discovery() -> Repeat<(ActivePeersList, CommandTx)> {
    Box::new(|(active_peers, command_tx)| {
        if let Some(peer_entry) = active_peers.read().oldest() {
            let peer_id = peer_entry.peer_id().clone();

            command_tx
                .send(Command::SendDiscoveryRequest { peer_id })
                .expect("error sending command");
        }
    })
}
