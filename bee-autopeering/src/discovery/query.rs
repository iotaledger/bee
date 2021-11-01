// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::protocol;

use crate::{
    command::{Command, CommandTx},
    discovery::manager,
    event::EventTx,
    local::Local,
    peer::{
        peerlist::{ActivePeerEntry, ActivePeersList, MasterPeersList, ReplacementList},
        peerstore::PeerStore,
        PeerId,
    },
    request::RequestManager,
    server::ServerTx,
    task::Repeat,
};

use rand::{thread_rng, Rng};
use tokio::sync::mpsc::Receiver;

use std::{collections::VecDeque, time::Duration};

#[derive(Clone)]
pub(crate) struct QueryContext<S: PeerStore> {
    pub(crate) local: Local,
    pub(crate) peerstore: S,
    pub(crate) request_mngr: RequestManager<S>,
    pub(crate) master_peers: MasterPeersList,
    pub(crate) active_peers: ActivePeersList,
    pub(crate) replacements: ReplacementList,
    pub(crate) server_tx: ServerTx,
    pub(crate) event_tx: EventTx,
}

// Hive.go: pings the oldest active peer.
pub(crate) fn do_reverify<S: PeerStore + 'static>() -> Repeat<QueryContext<S>> {
    Box::new(|ctx| {
        // Determine the next peer to re/verifiy.
        if let Some(active_peer) = peer_to_reverify(&ctx.active_peers) {
            log::debug!("Reverifying {}...", active_peer.peer_id());

            // CHANGE BACK: move to before tokio::spawn
            let ctx_ = ctx.clone();

            // TODO: introduce `UnsupervisedTask` type, that always finishes after a timeout.
            let _ = tokio::spawn(async move {
                if let Some(services) = manager::begin_verification_request(
                    active_peer.peer_id(),
                    &ctx_.request_mngr,
                    &ctx_.peerstore,
                    &ctx_.server_tx,
                )
                .await
                {
                    // Hive.go: no need to do anything here, as the peer is bumped when handling the pong
                    log::debug!("Reverification successful. Peer offers {} service/s.", services.len());
                } else {
                    log::debug!("Reverification failed. Removing peer {}.", active_peer.peer_id());

                    manager::remove_peer_from_active_list(
                        active_peer.peer_id(),
                        &ctx_.master_peers,
                        &ctx_.active_peers,
                        &ctx_.replacements,
                        &ctx_.peerstore,
                        &ctx_.event_tx,
                    )
                }
            });
        }
    })
}

// Hive.go: returns the oldest peer, or nil if empty.
fn peer_to_reverify(active_peers: &ActivePeersList) -> Option<ActivePeerEntry> {
    if active_peers.read().is_empty() {
        None
    } else {
        active_peers.read().get_oldest().cloned()
    }
}

// Hive.go:
// The current strategy is to always select the latest verified peer and one of
// the peers that returned the most number of peers the last time it was queried.
pub(crate) fn do_query<S: PeerStore + 'static>() -> Repeat<QueryContext<S>> {
    Box::new(|ctx| {
        let peers = select_peers_to_query(&ctx.active_peers);
        if peers.is_empty() {
            log::warn!("No peers to query.");
        } else {
            log::debug!("Querying {} peer/s...", peers.len());

            for peer_id in peers.into_iter() {
                let ctx_ = ctx.clone();

                // TODO: introduce `UnsupervisedTask` type, that always finishes after a timeout.
                tokio::spawn(async move {
                    if let Some(peers) =
                        manager::begin_discovery_request(&peer_id, &ctx_.request_mngr, &ctx_.peerstore, &ctx_.server_tx)
                            .await
                    {
                        log::debug!("Query successful. Received {} peers.", peers.len());

                        // Add the discovered peers to list and store
                        let mut num_added = 0;
                        for peer in peers {
                            if manager::add_peer(
                                peer,
                                &ctx_.local,
                                &ctx_.active_peers,
                                &ctx_.replacements,
                                &ctx_.peerstore,
                            ) {
                                num_added += 1;
                            }
                        }

                        // Remember how many new peers were discovered through the queried peer.
                        ctx_.active_peers
                            .write()
                            .find_mut(&peer_id)
                            .expect("inconsistent active peers list")
                            .metrics_mut()
                            .set_last_new_peers(num_added);
                    } else {
                        log::debug!("Query unsuccessful. Removing peer {}.", peer_id);

                        manager::remove_peer_from_active_list(
                            &peer_id,
                            &ctx_.master_peers,
                            &ctx_.active_peers,
                            &ctx_.replacements,
                            &ctx_.peerstore,
                            &ctx_.event_tx,
                        )
                    }
                });
            }
        }
    })
}

// Hive.go: selects the peers that should be queried.
fn select_peers_to_query(active_peers: &ActivePeersList) -> Vec<PeerId> {
    let mut verif_peers = protocol::get_verified_peers(active_peers);

    // If we have less than 3 verified peers, then we use those for the query.
    if verif_peers.len() < 3 {
        verif_peers
            .into_iter()
            .map(|ap| ap.peer_id().clone())
            .collect::<Vec<_>>()
    } else {
        // Note: this macro is useful to remove some noise from the pattern matching rules.
        macro_rules! num {
            ($t:expr) => {
                // Panic: we made sure, that unwrap is always okay.
                $t.as_ref().unwrap().metrics().last_new_peers()

                // TODO: remove this when pretty certain about the correctness of the rules.
                // if let Some(pe) = $t.as_ref() {
                //     $t.as_ref().unwrap().metrics().last_new_peers()
                // } else {
                //     255
                // }
            };
        }

        fn len<T>(o: &(Option<T>, Option<T>, Option<T>)) -> usize {
            let a = if o.0.is_some() { 1 } else { 0 };
            let b = if o.1.is_some() { 1 } else { 0 };
            let c = if o.2.is_some() { 1 } else { 0 };
            a + b + c
        }

        let latest = verif_peers.remove(0).peer_id().clone();

        // Note: This loop finds the three "heaviest" peers with one iteration over an unsorted vec of verified peers.
        let heaviest3 = verif_peers.into_iter().fold(
            (None, None, None),
            |(x, y, z): (
                Option<ActivePeerEntry>,
                Option<ActivePeerEntry>,
                Option<ActivePeerEntry>,
            ),
             p| {
                let n = p.metrics().last_new_peers();

                // TODO: remove this when pretty certain about the correctness of the rules.
                // println!(
                //     "{} {} {} --- {}",
                //     num!(x),
                //     num!(y),
                //     num!(z),
                //     p.metrics().last_new_peers()
                // );

                match (&x, &y, &z) {
                    // set 1st
                    (None, _, _) => (Some(p), y, z),
                    // shift-right + set 1st
                    (t, None, _) if n < num!(t) => (Some(p), t.clone(), z),
                    // set 2nd
                    (t, None, _) if n >= num!(t) => (x, Some(p), z),
                    // shift-right + shift-right + set 1st
                    (s, t, None) if n < num!(s) => (Some(p), s.clone(), t.clone()),
                    // shift-right + set 1st
                    (_, t, None) if n < num!(t) => (x, Some(p), t.clone()),
                    // set 3rd
                    (_, t, None) if n >= num!(t) => (x, y, Some(p)),
                    // no-op
                    (t, _, _) if n < num!(t) => (x, y, z),
                    // set 1st
                    (_, t, _) if n < num!(t) => (Some(p), y, z),
                    // shift-left + set 2nd
                    (_, _, t) if n < num!(t) => (y, Some(p), z),
                    // shift-left + shift-left + set 3rd
                    (_, _, _) => (y, z, Some(p)),
                }
            },
        );

        let r = thread_rng().gen_range(0..len(&heaviest3));
        let heaviest = match r {
            0 => heaviest3.0,
            1 => heaviest3.1,
            2 => heaviest3.2,
            _ => unreachable!(),
        }
        // Panic: we made sure that the unwrap is always possible.
        .unwrap()
        .peer_id()
        .clone();

        vec![latest, heaviest]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::{peerlist::ActivePeerEntry, Peer};

    fn create_peerlist_of_size(n: usize) -> ActivePeersList {
        // Create a set of active peer entries.
        let mut entries = (0..n)
            .map(|i| Peer::new_test_peer(i as u8))
            .map(|p| ActivePeerEntry::new(p))
            .collect::<Vec<_>>();

        // Create a peerlist, and insert the peer entries setting the `last_new_peers` metric
        // equal to its peerlist index. We also need to set the `verified_count` to at least 1.
        let peerlist = ActivePeersList::default();
        let mut pl = peerlist.write();
        for (i, mut entry) in entries.into_iter().enumerate() {
            entry.metrics_mut().set_last_new_peers((n - 1) - i);
            entry.metrics_mut().increment_verified_count();

            pl.insert(entry);
        }
        drop(pl);
        peerlist
    }

    #[test]
    fn find_peers_to_query_in_peerlist_1() {
        let peerlist = create_peerlist_of_size(1);

        let selected = select_peers_to_query(&peerlist);
        assert_eq!(1, selected.len());
    }

    #[test]
    fn find_peers_to_query_in_peerlist_2() {
        let peerlist = create_peerlist_of_size(2);

        let selected = select_peers_to_query(&peerlist);
        assert_eq!(2, selected.len());
    }

    #[test]
    fn find_peers_to_query_in_peerlist_3() {
        let peerlist = create_peerlist_of_size(3);

        macro_rules! equal {
            ($a:expr, $b:expr) => {{
                $a == peerlist.read().get($b).unwrap().peer_id()
            }};
        }

        let selected = select_peers_to_query(&peerlist);
        assert_eq!(2, selected.len());

        assert!(equal!(&selected[0], 0));
        assert!(equal!(&selected[1], 1) || equal!(&selected[1], 2));
    }

    #[test]
    fn find_peers_to_query_in_peerlist_10() {
        let peerlist = create_peerlist_of_size(10);

        macro_rules! equal {
            ($a:expr, $b:expr) => {{
                $a == peerlist.read().get($b).unwrap().peer_id()
            }};
        }

        // 0 1 2 3 4 ... 7 8 9 (index)
        // 0 1 2 3 4 ... 7 8 9 (last_new_peers)
        // ^             ^ ^ ^
        // 0             1 1 1 (expected)
        let selected = select_peers_to_query(&peerlist);
        assert_eq!(2, selected.len());

        // Always the newest peer (index 0) is selected.
        assert!(equal!(&selected[0], 0));
        // Either of the 3 "heaviest" peers is selected.
        assert!(equal!(&selected[1], 7) || equal!(&selected[1], 8) || equal!(&selected[1], 9));

        // 0 1 2 3 4 ... 7 8 9 (index)
        // 8 9 0 1 2 ... 5 6 7 (last_new_peers)
        // ^ ^             ^ ^
        // 0 1             1 1 (expected)
        peerlist.write().rotate_forwards();
        peerlist.write().rotate_forwards();

        let selected = select_peers_to_query(&peerlist);
        assert_eq!(2, selected.len());

        assert!(equal!(&selected[0], 0));
        assert!(equal!(&selected[1], 1) || equal!(&selected[1], 8) || equal!(&selected[1], 9));
    }
}
