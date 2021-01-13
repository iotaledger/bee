// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::model::Diff;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::{diff::rand_diff, milestone::rand_milestone_index};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/milestone_index_to_diff";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (index, diff) = (rand_milestone_index(), rand_diff());

    assert!(!Exist::<MilestoneIndex, Diff>::exist(&storage, &index).await.unwrap());
    assert!(Fetch::<MilestoneIndex, Diff>::fetch(&storage, &index)
        .await
        .unwrap()
        .is_none());

    Insert::<MilestoneIndex, Diff>::insert(&storage, &index, &diff)
        .await
        .unwrap();

    assert!(Exist::<MilestoneIndex, Diff>::exist(&storage, &index).await.unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, Diff>::fetch(&storage, &index)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        diff.pack_new()
    );

    Delete::<MilestoneIndex, Diff>::delete(&storage, &index).await.unwrap();

    assert!(!Exist::<MilestoneIndex, Diff>::exist(&storage, &index).await.unwrap());
    assert!(Fetch::<MilestoneIndex, Diff>::fetch(&storage, &index)
        .await
        .unwrap()
        .is_none());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (index, diff) = (rand_milestone_index(), rand_diff());
        Insert::<MilestoneIndex, Diff>::insert(&storage, &index, &diff)
            .await
            .unwrap();
        Batch::<MilestoneIndex, Diff>::batch_delete(&storage, &mut batch, &index).unwrap();
    }

    let mut diffs = HashMap::new();

    for _ in 0usize..10usize {
        let (index, diff) = (rand_milestone_index(), rand_diff());
        Batch::<MilestoneIndex, Diff>::batch_insert(&storage, &mut batch, &index, &diff).unwrap();
        diffs.insert(index, diff);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, Diff>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((index, diff)) = stream.next().await {
        assert_eq!(diffs.get(&index).unwrap().pack_new(), diff.pack_new());
        count += 1;
    }

    assert_eq!(count, diffs.len());

    Truncate::<MilestoneIndex, Diff>::truncate(&storage).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, Diff>::stream(&storage).await.unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
