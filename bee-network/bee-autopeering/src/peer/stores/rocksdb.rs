// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::peer::{
    lists::{ActivePeer, ActivePeersList, ReplacementPeersList},
    peer_id::PeerId,
    stores::PeerStore,
    Peer,
};

use rocksdb::{AsColumnFamilyRef, DBWithThreadMode, IteratorMode, MultiThreaded, Options, WriteBatch};

use std::{path::PathBuf, sync::Arc};

const ACTIVE_PEERS_CF: &str = "active_peers";
const REPLACEMENTS_CF: &str = "replacements";

/// The config for the RocksDb peer store.
#[derive(Clone)]
pub struct RocksDbPeerStoreConfig {
    path: PathBuf,
    options: Options,
}

/// The (persistent) RocksDb peer store.
#[derive(Clone)]
pub struct RocksDbPeerStore {
    db: Arc<DBWithThreadMode<MultiThreaded>>,
    config: RocksDbPeerStoreConfig,
}

impl RocksDbPeerStore {
    fn open_cf(&self, cf_str: &'static str) -> impl AsColumnFamilyRef + '_ {
        self.db.cf_handle(cf_str).expect("error opening cf")
    }
}

impl PeerStore for RocksDbPeerStore {
    type Config = RocksDbPeerStoreConfig;

    fn new(config: Self::Config) -> Self {
        let db = DBWithThreadMode::open_cf(&config.options, &config.path, &[ACTIVE_PEERS_CF, REPLACEMENTS_CF])
            .expect("error opening peer store");

        Self {
            db: Arc::new(db),
            config,
        }
    }

    fn store_active(&self, active_peer: ActivePeer) {
        let cf = self.open_cf(ACTIVE_PEERS_CF);
        let key = *active_peer.peer_id();

        self.db.put_cf(&cf, key, active_peer.to_bytes()).expect("insert error");
    }

    fn store_all_active(&self, active_peers: &ActivePeersList) {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        let mut batch = WriteBatch::default();
        active_peers
            .read()
            .iter()
            .for_each(|p| batch.put_cf(&cf, p.peer_id(), p.to_bytes()));

        self.db.write(batch).expect("error applying batch");
    }

    fn store_replacement(&self, peer: Peer) {
        let cf = self.open_cf(REPLACEMENTS_CF);
        let key = *peer.peer_id();

        self.db.put_cf(&cf, key, peer.to_bytes()).expect("error inserting peer");
    }

    fn store_all_replacements(&self, replacements: &ReplacementPeersList) {
        let cf = self.open_cf(REPLACEMENTS_CF);

        let mut batch = WriteBatch::default();
        replacements
            .read()
            .iter()
            .for_each(|p| batch.put_cf(&cf, p.peer_id(), p.to_bytes()));

        self.db.write(batch).expect("error applying batch");
    }

    fn contains(&self, peer_id: &PeerId) -> bool {
        let cf = self.open_cf(ACTIVE_PEERS_CF);
        if self.db.get_cf(&cf, peer_id).expect("db error").is_some() {
            true
        } else {
            let cf = self.open_cf(REPLACEMENTS_CF);
            self.db.get_cf(&cf, peer_id).expect("db error").is_some()
        }
    }

    fn fetch_active(&self, peer_id: &PeerId) -> Option<ActivePeer> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        self.db
            .get_cf(&cf, peer_id)
            .expect("db error")
            .map(|b| ActivePeer::from_bytes(&b))
    }

    fn fetch_all_active(&self) -> Vec<ActivePeer> {
        let cf = self.open_cf(ACTIVE_PEERS_CF);

        self.db
            .iterator_cf(&cf, IteratorMode::Start)
            .map(|(_, b)| ActivePeer::from_bytes(&b))
            .collect::<Vec<_>>()
    }

    fn fetch_replacement(&self, peer_id: &PeerId) -> Option<Peer> {
        let cf = self.open_cf(REPLACEMENTS_CF);

        self.db.get_cf(&cf, peer_id).expect("db error").map(Peer::from_bytes)
    }

    fn fetch_all_replacements(&self) -> Vec<Peer> {
        let cf = self.open_cf(REPLACEMENTS_CF);

        self.db
            .iterator_cf(&cf, IteratorMode::Start)
            .map(|(_, bytes)| Peer::from_bytes(bytes))
            .collect::<Vec<_>>()
    }

    fn delete(&self, _: &PeerId) -> bool {
        unimplemented!("no need for single entry removal at the moment")
    }

    fn delete_all(&self) {
        self.db.drop_cf(ACTIVE_PEERS_CF).expect("error dropping cf");
        self.db
            .create_cf(ACTIVE_PEERS_CF, &self.config.options)
            .expect("error creating cf");

        self.db.drop_cf(REPLACEMENTS_CF).expect("error dropping cf");
        self.db
            .create_cf(REPLACEMENTS_CF, &self.config.options)
            .expect("error creating cf");
    }
}
