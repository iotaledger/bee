// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::{Milestone, MilestoneIndex};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::milestone::{random_milestone, random_milestone_index};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/milestone_index_to_milestone";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let index = random_milestone_index();
    let milestone = random_milestone();

    assert!(!Exist::<MilestoneIndex, Milestone>::exist(&storage, &index)
        .await
        .unwrap());
    assert!(Fetch::<MilestoneIndex, Milestone>::fetch(&storage, &index)
        .await
        .unwrap()
        .is_none());

    Insert::<MilestoneIndex, Milestone>::insert(&storage, &index, &milestone)
        .await
        .unwrap();

    assert!(Exist::<MilestoneIndex, Milestone>::exist(&storage, &index)
        .await
        .unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, Milestone>::fetch(&storage, &index)
            .await
            .unwrap()
            .unwrap(),
        milestone
    );

    Delete::<MilestoneIndex, Milestone>::delete(&storage, &index)
        .await
        .unwrap();

    assert!(!Exist::<MilestoneIndex, Milestone>::exist(&storage, &index)
        .await
        .unwrap());
    assert!(Fetch::<MilestoneIndex, Milestone>::fetch(&storage, &index)
        .await
        .unwrap()
        .is_none());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (index, milestone) = (random_milestone_index(), random_milestone());
        Insert::<MilestoneIndex, Milestone>::insert(&storage, &index, &milestone)
            .await
            .unwrap();
        Batch::<MilestoneIndex, Milestone>::batch_delete(&storage, &mut batch, &index).unwrap();
    }

    let mut milestones = HashMap::new();

    for _ in 0usize..10usize {
        let (index, milestone) = (random_milestone_index(), random_milestone());
        Batch::<MilestoneIndex, Milestone>::batch_insert(&storage, &mut batch, &index, &milestone).unwrap();
        milestones.insert(index, milestone);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((index, milestone)) = stream.next().await {
        assert_eq!(milestones.get(&index).unwrap(), &milestone);
        count += 1;
    }

    assert_eq!(count, milestones.len());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
