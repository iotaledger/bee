// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// TODO: logic to reconnect a known peer once disconnected


// // Automatically try to reconnect known **and** discovered peers. The removal of discovered peers is a decision
// // that needs to be made in the autopeering service.
// for (peer_id, info) in peer_state_map.filter_info(|info, state| {
//     (info.relation.is_known() || info.relation.is_discovered()) && state.is_disconnected()
// }) {
//     log::debug!("Trying to reconnect to: {} ({peer_id}).", info.alias);

//     // Ignore if the command fails. We can always retry the next time.
//     let _ = senders
//         .server_command_tx
//         .send(GossipManagerCommand::ConnectPeer { peer_id });
// }
