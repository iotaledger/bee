// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Unspent;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDbConfigBuilder, storage::Storage};
use bee_test::rand::output::rand_unspent_output_id;

use futures::stream::StreamExt;

const DB_DIRECTORY: &str = "./tests/database/output_id_unspent";

#[tokio::test]
async fn output_id_unspent_access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDbConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let unspent = rand_unspent_output_id();

    assert!(!Exist::<Unspent, ()>::exist(&storage, &unspent).await.unwrap());

    Insert::<Unspent, ()>::insert(&storage, &unspent, &()).await.unwrap();

    assert!(Exist::<Unspent, ()>::exist(&storage, &unspent).await.unwrap());

    Delete::<Unspent, ()>::delete(&storage, &unspent).await.unwrap();

    assert!(!Exist::<Unspent, ()>::exist(&storage, &unspent).await.unwrap());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let unspent = rand_unspent_output_id();
        Insert::<Unspent, ()>::insert(&storage, &unspent, &()).await.unwrap();
        Batch::<Unspent, ()>::batch_delete(&storage, &mut batch, &unspent).unwrap();
    }

    let mut unspents = Vec::new();

    for _ in 0usize..10usize {
        let unspent = rand_unspent_output_id();
        Batch::<Unspent, ()>::batch_insert(&storage, &mut batch, &unspent, &()).unwrap();
        unspents.push(unspent);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<Unspent, ()>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((unspent, ())) = stream.next().await {
        assert!(unspents.contains(&unspent));
        count += 1;
    }

    assert_eq!(count, unspents.len());

    Truncate::<Unspent, ()>::truncate(&storage).await.unwrap();

    let mut stream = AsStream::<Unspent, ()>::stream(&storage).await.unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
