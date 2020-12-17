// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::LedgerIndex;
use bee_protocol::MilestoneIndex;
use bee_storage::{
    access::{Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};

const DB_DIRECTORY: &str = "./tests/database/ledger_index";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let index_1 = LedgerIndex::from(MilestoneIndex::from(42));

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(&storage, &(), &index_1)
        .await
        .unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());

    let index_2 = Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().unwrap();

    assert_eq!(index_1, index_2);

    Delete::<(), LedgerIndex>::delete(&storage, &()).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());

    let mut batch = Storage::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(&storage, &mut batch, &(), &index_1).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());

    let index_2 = Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().unwrap();

    assert_eq!(index_1, index_2);

    let mut batch = Storage::batch_begin();

    Batch::<(), LedgerIndex>::batch_delete(&storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());
}
