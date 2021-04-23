// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::milestone::{rand_milestone, rand_milestone_index};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneIndex, Milestone>
    + Fetch<MilestoneIndex, Milestone>
    + Insert<MilestoneIndex, Milestone>
    + Delete<MilestoneIndex, Milestone>
    + BatchBuilder
    + Batch<MilestoneIndex, Milestone>
    + for<'a> AsStream<'a, MilestoneIndex, Milestone>
    + Truncate<MilestoneIndex, Milestone>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneIndex, Milestone>
        + Fetch<MilestoneIndex, Milestone>
        + Insert<MilestoneIndex, Milestone>
        + Delete<MilestoneIndex, Milestone>
        + BatchBuilder
        + Batch<MilestoneIndex, Milestone>
        + for<'a> AsStream<'a, MilestoneIndex, Milestone>
        + Truncate<MilestoneIndex, Milestone>
{
}

pub async fn milestone_index_to_milestone_access<B: StorageBackend>(storage: &B) {
    let (index, milestone) = (rand_milestone_index(), rand_milestone());

    assert!(
        !Exist::<MilestoneIndex, Milestone>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .await
            .unwrap()
            .is_none()
    );

    Insert::<MilestoneIndex, Milestone>::insert(storage, &index, &milestone)
        .await
        .unwrap();

    assert!(
        Exist::<MilestoneIndex, Milestone>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .await
            .unwrap()
            .unwrap(),
        milestone
    );

    Delete::<MilestoneIndex, Milestone>::delete(storage, &index)
        .await
        .unwrap();

    assert!(
        !Exist::<MilestoneIndex, Milestone>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        Insert::<MilestoneIndex, Milestone>::insert(storage, &index, &milestone)
            .await
            .unwrap();
        Batch::<MilestoneIndex, Milestone>::batch_delete(storage, &mut batch, &index).unwrap();
    }

    let mut milestones = HashMap::new();

    for _ in 0usize..10usize {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        Batch::<MilestoneIndex, Milestone>::batch_insert(storage, &mut batch, &index, &milestone).unwrap();
        milestones.insert(index, milestone);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((index, milestone)) = stream.next().await {
        assert_eq!(milestones.get(&index).unwrap(), &milestone);
        count += 1;
    }

    assert_eq!(count, milestones.len());

    Truncate::<MilestoneIndex, Milestone>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, Milestone>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
