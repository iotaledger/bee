// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::model::LedgerIndex;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};

use futures::stream::StreamExt;

const DB_DIRECTORY: &str = "./tests/database/ledger_index";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let index = LedgerIndex::from(MilestoneIndex::from(42));

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(&storage, &(), &index).await.unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().unwrap(),
        index
    );

    Delete::<(), LedgerIndex>::delete(&storage, &()).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());

    let mut batch = Storage::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(&storage, &mut batch, &(), &index).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().unwrap(),
        index
    );

    let mut batch = Storage::batch_begin();

    Batch::<(), LedgerIndex>::batch_delete(&storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(&storage, &()).await.unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(&storage, &(), &index).await.unwrap();

    let mut stream = AsStream::<(), LedgerIndex>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((_, ledger_index)) = stream.next().await {
        assert_eq!(ledger_index, index);
        count += 1;
    }

    assert_eq!(count, 1);

    Truncate::<(), LedgerIndex>::truncate(&storage).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(&storage, &()).await.unwrap());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
