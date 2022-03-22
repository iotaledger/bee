// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use sled::{Batch, Db};

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
};

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

    type Error = sled::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        let db = config.open()?;

        db.open_tree(ACTIVE_PEERS_TREE)?;
        db.open_tree(REPLACEMENTS_TREE)?;

        Ok(Self { db })
    }

    fn store_active(&self, active_peer: ActivePeer) -> Result<(), Self::Error> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE)?;
        let key = *active_peer.peer_id();

        tree.insert(key, active_peer.to_bytes())?;

        Ok(())
    }

    fn store_all_active(&self, active_peers: &ActivePeersList) -> Result<(), Self::Error> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE)?;

        let mut batch = Batch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id(), p.clone()));

        tree.apply_batch(batch)?;

        Ok(())
    }

    fn store_replacement(&self, peer: Peer) -> Result<(), Self::Error> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE)?;
        let key = *peer.peer_id();

        tree.insert(key, peer)?;

        Ok(())
    }

    fn store_all_replacements(&self, replacements: &ReplacementPeersList) -> Result<(), Self::Error> {
        let replacements_tree = self.db.open_tree(REPLACEMENTS_TREE)?;

        let mut batch = Batch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.insert(p.peer_id(), p.clone()));

        replacements_tree.apply_batch(batch)?;

        Ok(())
    }

    fn contains(&self, peer_id: &PeerId) -> Result<bool, Self::Error> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE)?;
        if tree.contains_key(peer_id)? {
            Ok(true)
        } else {
            let tree = self.db.open_tree(REPLACEMENTS_TREE)?;
            tree.contains_key(peer_id)
        }
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Result<Option<ActivePeer>, Self::Error> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE)?;

        Ok(tree.get(peer_id)?.map(|b| ActivePeer::from_bytes(&b)))
    }

    fn fetch_all_active(&self) -> Result<Vec<ActivePeer>, Self::Error> {
        let tree = self.db.open_tree(ACTIVE_PEERS_TREE)?;

        tree.iter()
            .map(|r| r.map(|(_, b)| ActivePeer::from_bytes(&b)))
            .collect::<Result<Vec<_>, _>>()
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Result<Option<Peer>, Self::Error> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE)?;

        Ok(tree.get(peer_id)?.map(Peer::from))
    }

    fn fetch_all_replacements(&self) -> Result<Vec<Peer>, Self::Error> {
        let tree = self.db.open_tree(REPLACEMENTS_TREE)?;

        tree.iter()
            .map(|r| r.map(|(_, ivec)| Peer::from(ivec)))
            .collect::<Result<Vec<_>, _>>()
    }

    fn delete(&self, _: &PeerId) -> Result<bool, Self::Error> {
        unimplemented!("no need for single entry removal at the moment")
    }

    fn delete_all(&self) -> Result<(), Self::Error> {
        self.db.open_tree(ACTIVE_PEERS_TREE)?.clear()?;
        self.db.open_tree(REPLACEMENTS_TREE)?.clear()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temporary_sled_peer_store() -> SledPeerStore {
        let config = SledPeerStoreConfig::new().temporary(true);
        SledPeerStore::new(config).unwrap()
    }

    #[test]
    fn store_and_fetch_active_peer() {
        let peer_store = create_temporary_sled_peer_store();

        let peer = ActivePeer::new(Peer::new_test_peer(0));
        let peer_id = *peer.peer_id();

        peer_store.store_active(peer).unwrap();

        let fetched_active_peer = peer_store.fetch_active(&peer_id).unwrap().expect("missing peer");

        assert_eq!(peer_id, *fetched_active_peer.peer_id());
    }

    #[test]
    fn store_and_fetch_replacement_peer() {
        let peer_store = create_temporary_sled_peer_store();

        let peer = Peer::new_test_peer(0);
        let peer_id = *peer.peer_id();

        peer_store.store_replacement(peer).unwrap();

        let fetched_peer = peer_store.fetch_replacement(&peer_id).unwrap().expect("missing peer");

        assert_eq!(peer_id, *fetched_peer.peer_id());
    }
}
