// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub use rocksdb::Options;
use rocksdb::{AsColumnFamilyRef, DBWithThreadMode, IteratorMode, MultiThreaded, WriteBatch};

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
};

const ACTIVE_PEERS_CF: &str = "active_peers";
const REPLACEMENTS_CF: &str = "replacements";

/// The config for the RocksDB peer store.
#[derive(Clone)]
pub struct RocksDbPeerStoreConfig {
    path: PathBuf,
    options: Options,
}

impl RocksDbPeerStoreConfig {
    /// Creates a new config for the RocksDB peer store.
    pub fn new<P: AsRef<Path>>(path: P, options: Options) -> Self {
        Self {
            path: path.as_ref().to_owned(),
            options,
        }
    }
}

/// The (persistent) RocksDb peer store.
#[derive(Clone)]
pub struct RocksDbPeerStore {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
}

impl RocksDbPeerStore {
    fn open_cf(&self, cf_str: &'static str) -> impl AsColumnFamilyRef + '_ {
        self.db.cf_handle(cf_str).unwrap()
    }
}

impl PeerStore for RocksDbPeerStore {
    type Config = RocksDbPeerStoreConfig;

    type Error = rocksdb::Error;

    fn new(config: Self::Config) -> Result<Self, Self::Error> {
        let db = DBWithThreadMode::open_cf(&config.options, &config.path, &[ACTIVE_PEERS_CF, REPLACEMENTS_CF])?;

        Ok(Self { db: Arc::new(db) })
    }

    fn store_active(&self, active_peer: ActivePeer) -> Result<(), Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);
        let key = *active_peer.peer_id();

        self.db.put_cf(&cf, key, active_peer.to_bytes())
    }

    fn store_all_active(&self, active_peers: &ActivePeersList) -> Result<(), Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        let mut batch = WriteBatch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.put_cf(&cf, p.peer_id(), p.to_bytes()));

        self.db.write(batch)
    }

    fn store_replacement(&self, peer: Peer) -> Result<(), Self::Error> {
        let cf = self.open_cf(REPLACEMENTS_CF);
        let key = *peer.peer_id();

        self.db.put_cf(&cf, key, peer.to_bytes())
    }

    fn store_all_replacements(&self, replacements: &ReplacementPeersList) -> Result<(), Self::Error> {
        let cf = self.open_cf(REPLACEMENTS_CF);

        let mut batch = WriteBatch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.put_cf(&cf, p.peer_id(), p.to_bytes()));

        self.db.write(batch)
    }

    fn contains(&self, peer_id: &PeerId) -> Result<bool, Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);
        if self.db.get_pinned_cf(&cf, peer_id)?.is_some() {
            Ok(true)
        } else {
            let cf = self.open_cf(REPLACEMENTS_CF);
            Ok(self.db.get_pinned_cf(&cf, peer_id)?.is_some())
        }
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Result<Option<ActivePeer>, Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        Ok(self.db.get_pinned_cf(&cf, peer_id)?.map(|b| ActivePeer::from_bytes(&b)))
    }

    fn fetch_all_active(&self) -> Result<Vec<ActivePeer>, Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        Ok(self
            .db
            .iterator_cf(&cf, IteratorMode::Start)
            .map(|(_, b)| ActivePeer::from_bytes(&b))
            .collect::<Vec<_>>())
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Result<Option<Peer>, Self::Error> {
        let cf = self.open_cf(REPLACEMENTS_CF);

        Ok(self.db.get_cf(&cf, peer_id)?.map(Peer::from_bytes))
    }

    fn fetch_all_replacements(&self) -> Result<Vec<Peer>, Self::Error> {
        let cf = self.open_cf(REPLACEMENTS_CF);

        Ok(self
            .db
            .iterator_cf(&cf, IteratorMode::Start)
            .map(|(_, bytes)| Peer::from_bytes(bytes))
            .collect::<Vec<_>>())
    }

    fn delete(&self, _: &PeerId) -> Result<bool, Self::Error> {
        unimplemented!("no need for single entry removal at the moment")
    }

    fn delete_all(&self) -> Result<(), Self::Error> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);
        self.db.delete_range_cf(&cf, [0; 32], [0xff; 32])?;

        let cf = self.open_cf(REPLACEMENTS_CF);
        self.db.delete_range_cf(&cf, [0; 32], [0xff; 32])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    struct Janitor<P: AsRef<Path>>(P);

    impl<P: AsRef<Path>> Drop for Janitor<P> {
        fn drop(&mut self) {
            if let Err(e) = std::fs::remove_dir_all(self.0.as_ref()) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    panic!("{}", e);
                }
            }
        }
    }

    use super::*;

    fn run_with_peer_store_in_path<P: AsRef<Path> + Copy>(path: P, f: fn(RocksDbPeerStore)) {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let config = RocksDbPeerStoreConfig::new(path, options);
        let peer_store = RocksDbPeerStore::new(config).unwrap();
        let janitor = Janitor(path);
        f(peer_store);
        drop(janitor);
    }

    #[test]
    fn store_and_fetch_active_peer() {
        fn f(peer_store: RocksDbPeerStore) {
            let peer = ActivePeer::new(Peer::new_test_peer(0));
            let peer_id = *peer.peer_id();

            peer_store.store_active(peer).unwrap();

            let fetched_active_peer = peer_store.fetch_active(&peer_id).unwrap().expect("missing peer");

            assert_eq!(peer_id, *fetched_active_peer.peer_id());
        }

        run_with_peer_store_in_path("rocksdb_store_and_fetch_active_peer", f)
    }

    #[test]
    fn store_and_fetch_replacement_peer() {
        fn f(peer_store: RocksDbPeerStore) {
            let peer = Peer::new_test_peer(0);
            let peer_id = *peer.peer_id();

            peer_store.store_replacement(peer).unwrap();

            let fetched_peer = peer_store.fetch_replacement(&peer_id).unwrap().expect("missing peer");

            assert_eq!(peer_id, *fetched_peer.peer_id());
        }

        run_with_peer_store_in_path("rocksdb_store_and_fetch_replacement_peer", f);
    }

    #[test]
    fn store_and_delete_all() {
        fn f(peer_store: RocksDbPeerStore) {
            let peer = ActivePeer::new(Peer::new_test_peer(0));
            let peer_id = *peer.peer_id();

            peer_store.store_active(peer).unwrap();

            peer_store.delete_all().unwrap();

            assert!(!peer_store.contains(&peer_id).unwrap());
        }

        run_with_peer_store_in_path("rocksdb_store_and_delete_all", f)
    }
}
