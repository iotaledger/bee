// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::{tangle::SolidEntryPoint, MilestoneIndex};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::{milestone::rand_milestone_index, solid_entry_point::rand_solid_entry_point};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/solid_entry_point_to_milestone_index";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());

    assert!(!Exist::<SolidEntryPoint, MilestoneIndex>::exist(&storage, &sep)
        .await
        .unwrap());
    assert!(Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(&storage, &sep)
        .await
        .unwrap()
        .is_none());

    Insert::<SolidEntryPoint, MilestoneIndex>::insert(&storage, &sep, &index)
        .await
        .unwrap();

    assert!(Exist::<SolidEntryPoint, MilestoneIndex>::exist(&storage, &sep)
        .await
        .unwrap());
    assert_eq!(
        Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(&storage, &sep)
            .await
            .unwrap()
            .unwrap(),
        index
    );

    Delete::<SolidEntryPoint, MilestoneIndex>::delete(&storage, &sep)
        .await
        .unwrap();

    assert!(!Exist::<SolidEntryPoint, MilestoneIndex>::exist(&storage, &sep)
        .await
        .unwrap());
    assert!(Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(&storage, &sep)
        .await
        .unwrap()
        .is_none());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(&storage, &sep, &index)
            .await
            .unwrap();
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete(&storage, &mut batch, &sep).unwrap();
    }

    let mut seps = HashMap::new();

    for _ in 0usize..10usize {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(&storage, &mut batch, &sep, &index).unwrap();
        seps.insert(sep, index);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some((sep, index)) = stream.next().await {
        assert_eq!(*seps.get(&sep).unwrap(), index);
        count += 1;
    }

    assert_eq!(count, seps.len());

    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(&storage)
        .await
        .unwrap();

    let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
