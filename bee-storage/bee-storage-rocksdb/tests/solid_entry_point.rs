// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::tangle::SolidEntryPoint;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::solid_entry_point::rand_solid_entry_point;

use futures::stream::StreamExt;

use std::collections::HashSet;

const DB_DIRECTORY: &str = "./tests/database/solid_entry_point";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let sep = rand_solid_entry_point();

    assert!(!Exist::<SolidEntryPoint, ()>::exist(&storage, &sep).await.unwrap());

    Insert::<SolidEntryPoint, ()>::insert(&storage, &sep, &())
        .await
        .unwrap();

    assert!(Exist::<SolidEntryPoint, ()>::exist(&storage, &sep).await.unwrap());

    Delete::<SolidEntryPoint, ()>::delete(&storage, &sep).await.unwrap();

    assert!(!Exist::<SolidEntryPoint, ()>::exist(&storage, &sep).await.unwrap());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let sep = rand_solid_entry_point();
        Insert::<SolidEntryPoint, ()>::insert(&storage, &sep, &())
            .await
            .unwrap();
        Batch::<SolidEntryPoint, ()>::batch_delete(&storage, &mut batch, &sep).unwrap();
    }

    let mut seps = HashSet::new();

    for _ in 0usize..10usize {
        let sep = rand_solid_entry_point();
        Batch::<SolidEntryPoint, ()>::batch_insert(&storage, &mut batch, &sep, &()).unwrap();
        seps.insert(sep);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<SolidEntryPoint, ()>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((sep, _)) = stream.next().await {
        assert!(seps.contains(&sep));
        count += 1;
    }

    assert_eq!(count, seps.len());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
