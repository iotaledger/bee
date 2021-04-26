// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::solid_entry_point::SolidEntryPoint;
use bee_test::rand::{milestone::rand_milestone_index, solid_entry_point::rand_solid_entry_point};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<SolidEntryPoint, MilestoneIndex>
    + Fetch<SolidEntryPoint, MilestoneIndex>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Delete<SolidEntryPoint, MilestoneIndex>
    + BatchBuilder
    + Batch<SolidEntryPoint, MilestoneIndex>
    + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
    + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<SolidEntryPoint, MilestoneIndex>
        + Fetch<SolidEntryPoint, MilestoneIndex>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Delete<SolidEntryPoint, MilestoneIndex>
        + BatchBuilder
        + Batch<SolidEntryPoint, MilestoneIndex>
        + for<'a> AsStream<'a, SolidEntryPoint, MilestoneIndex>
        + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

pub async fn solid_entry_point_to_milestone_index_access<B: StorageBackend>(storage: &B) {
    let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());

    assert!(
        !Exist::<SolidEntryPoint, MilestoneIndex>::exist(storage, &sep)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(storage, &sep)
            .await
            .unwrap()
            .is_none()
    );

    Insert::<SolidEntryPoint, MilestoneIndex>::insert(storage, &sep, &index)
        .await
        .unwrap();

    assert!(
        Exist::<SolidEntryPoint, MilestoneIndex>::exist(storage, &sep)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(storage, &sep)
            .await
            .unwrap()
            .unwrap(),
        index
    );

    Delete::<SolidEntryPoint, MilestoneIndex>::delete(storage, &sep)
        .await
        .unwrap();

    assert!(
        !Exist::<SolidEntryPoint, MilestoneIndex>::exist(storage, &sep)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<SolidEntryPoint, MilestoneIndex>::fetch(storage, &sep)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(storage, &sep, &index)
            .await
            .unwrap();
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete(storage, &mut batch, &sep).unwrap();
    }

    let mut seps = HashMap::new();

    for _ in 0..10 {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(storage, &mut batch, &sep, &index).unwrap();
        seps.insert(sep, index);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some((sep, index)) = stream.next().await {
        assert_eq!(*seps.get(&sep).unwrap(), index);
        count += 1;
    }

    assert_eq!(count, seps.len());

    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .unwrap();

    let mut stream = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());
}
