// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
};

use sled::{Batch, Db};

const ACTIVE_PEERS_TREE: &str = "active_peers";
const REPLACEMENTS_TREE: &str = "replacements";

/// The config for the Sled peer store.
pub type SledPeerStoreConfig = sled::Config;

/// The (persistent) Sled peer store.
#[derive(Clone)]
pub struct SledPeerStore {
    db: Db,
}

impl PeerStore for SledPeerStore {
    type Config = SledPeerStoreConfig;

    fn new(config: Self::Config) -> Self {
        let db = config.open().expect("error opening peer store");

        db.open_tree("active_peers").expect("error opening tree");
        db.open_tree("replacements").expect("error opening tree");

        Self { db }
    }

    fn store_active(&self, active_peer: ActivePeer) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        let key = *active_peer.peer_id();

        tree.insert(key, active_peer.to_bytes()).expect("insert error");
    }

    fn store_all_active(&self, active_peers: &ActivePeersList) {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id(), p.clone()));

        tree.apply_batch(batch).expect("error applying batch");
    }

    fn store_replacement(&self, peer: Peer) {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
        let key = *peer.peer_id();

        tree.insert(key, peer).expect("error inserting peer");
    }

    fn store_all_replacements(&self, replacements: &ReplacementPeersList) {
        let replacements_tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        let mut batch = Batch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id(), p.clone()));

        replacements_tree.apply_batch(batch).expect("error applying batch");
    }

    fn contains(&self, peer_id: &PeerId) -> bool {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");
        if tree.contains_key(peer_id).expect("db error") {
            true
        } else {
            let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");
            tree.contains_key(peer_id).expect("db error")
        }
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        tree.get(peer_id).expect("db error").map(|b| ActivePeer::from_bytes(&b))
    }

    fn fetch_all_active(&self) -> Vec<ActivePeer> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE).expect("error opening tree");

        tree.iter()
            .filter_map(|p| p.ok())
            .map(|(_, b)| ActivePeer::from_bytes(&b))
            .collect::<Vec<_>>()
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        tree.get(peer_id).expect("db error").map(Peer::from)
    }

    fn fetch_all_replacements(&self) -> Vec<Peer> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE).expect("error opening tree");

        tree.iter()
            .filter_map(|p| p.ok())
            .map(|(_, ivec)| Peer::from(ivec))
            .collect::<Vec<_>>()
    }

    fn delete(&self, _: &PeerId) -> bool {
        unimplemented!("no need for single entry removal at the moment")
    }

    fn delete_all(&self) {
        self.db
            .open_tree(ACTIVE_PEERS_TREE)
            .expect("error opening tree")
            .clear()
            .expect("error clearing tree");

        self.db
            .open_tree(REPLACEMENTS_TREE)
            .expect("error opening tree")
            .clear()
            .expect("error clearing tree");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temporary_sled_peer_store() -> SledPeerStore {
        let config = SledPeerStoreConfig::new().temporary(true);
        SledPeerStore::new(config)
    }

    #[test]
    fn store_and_fetch_active_peer() {
        let peer_store = create_temporary_sled_peer_store();

        let peer = ActivePeer::new(Peer::new_test_peer(0));
        let peer_id = *peer.peer_id();

        peer_store.store_active(peer);

        let fetched_active_peer = peer_store.fetch_active(&peer_id).expect("missing peer");

        assert_eq!(peer_id, *fetched_active_peer.peer_id());
    }

    #[test]
    fn store_and_fetch_replacement_peer() {
        let peer_store = create_temporary_sled_peer_store();

        let peer = Peer::new_test_peer(0);
        let peer_id = *peer.peer_id();

        peer_store.store_replacement(peer);

        let fetched_peer = peer_store.fetch_replacement(&peer_id).expect("missing peer");

        assert_eq!(peer_id, *fetched_peer.peer_id());
    }
}
