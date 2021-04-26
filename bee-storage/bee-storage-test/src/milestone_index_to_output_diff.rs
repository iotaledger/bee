// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{milestone::rand_milestone_index, output_diff::rand_output_diff};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneIndex, OutputDiff>
    + Fetch<MilestoneIndex, OutputDiff>
    + Insert<MilestoneIndex, OutputDiff>
    + Delete<MilestoneIndex, OutputDiff>
    + BatchBuilder
    + Batch<MilestoneIndex, OutputDiff>
    + for<'a> AsStream<'a, MilestoneIndex, OutputDiff>
    + Truncate<MilestoneIndex, OutputDiff>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, OutputDiff>
        + Insert<MilestoneIndex, OutputDiff>
        + Delete<MilestoneIndex, OutputDiff>
        + BatchBuilder
        + Batch<MilestoneIndex, OutputDiff>
        + for<'a> AsStream<'a, MilestoneIndex, OutputDiff>
        + Truncate<MilestoneIndex, OutputDiff>
{
}

pub async fn milestone_index_to_output_diff_access<B: StorageBackend>(storage: &B) {
    let (index, output_diff) = (rand_milestone_index(), rand_output_diff());

    assert!(
        !Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .await
            .unwrap()
            .is_none()
    );

    Insert::<MilestoneIndex, OutputDiff>::insert(storage, &index, &output_diff)
        .await
        .unwrap();

    assert!(
        Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        output_diff.pack_new()
    );

    Delete::<MilestoneIndex, OutputDiff>::delete(storage, &index)
        .await
        .unwrap();

    assert!(
        !Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (index, output_diff) = (rand_milestone_index(), rand_output_diff());
        Insert::<MilestoneIndex, OutputDiff>::insert(storage, &index, &output_diff)
            .await
            .unwrap();
        Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, &mut batch, &index).unwrap();
    }

    let mut output_diffs = HashMap::new();

    for _ in 0usize..10usize {
        let (index, output_diff) = (rand_milestone_index(), rand_output_diff());
        Batch::<MilestoneIndex, OutputDiff>::batch_insert(storage, &mut batch, &index, &output_diff).unwrap();
        output_diffs.insert(index, output_diff);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, OutputDiff>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((index, output_diff)) = stream.next().await {
        assert_eq!(output_diffs.get(&index).unwrap().pack_new(), output_diff.pack_new());
        count += 1;
    }

    assert_eq!(count, output_diffs.len());

    Truncate::<MilestoneIndex, OutputDiff>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<MilestoneIndex, OutputDiff>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
