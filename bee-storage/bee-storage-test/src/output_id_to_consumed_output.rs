// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::ConsumedOutput;
use bee_message::output::OutputId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::output::{rand_consumed_output, rand_output_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, ConsumedOutput>
    + Fetch<OutputId, ConsumedOutput>
    + Insert<OutputId, ConsumedOutput>
    + Delete<OutputId, ConsumedOutput>
    + BatchBuilder
    + Batch<OutputId, ConsumedOutput>
    + for<'a> AsStream<'a, OutputId, ConsumedOutput>
    + Truncate<OutputId, ConsumedOutput>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<OutputId, ConsumedOutput>
        + Fetch<OutputId, ConsumedOutput>
        + Insert<OutputId, ConsumedOutput>
        + Delete<OutputId, ConsumedOutput>
        + BatchBuilder
        + Batch<OutputId, ConsumedOutput>
        + for<'a> AsStream<'a, OutputId, ConsumedOutput>
        + Truncate<OutputId, ConsumedOutput>
{
}

pub async fn output_id_to_consumed_output_access<B: StorageBackend>(storage: &B) {
    let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());

    assert!(!Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert!(Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
        .await
        .unwrap()
        .is_none());

    Insert::<OutputId, ConsumedOutput>::insert(storage, &output_id, &consumed_output)
        .await
        .unwrap();

    assert!(Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert_eq!(
        Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .unwrap(),
        consumed_output
    );

    Delete::<OutputId, ConsumedOutput>::delete(storage, &output_id)
        .await
        .unwrap();

    assert!(!Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert!(Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
        .await
        .unwrap()
        .is_none());

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        Insert::<OutputId, ConsumedOutput>::insert(storage, &output_id, &consumed_output)
            .await
            .unwrap();
        Batch::<OutputId, ConsumedOutput>::batch_delete(storage, &mut batch, &output_id).unwrap();
    }

    let mut consumed_outputs = HashMap::new();

    for _ in 0usize..10usize {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        Batch::<OutputId, ConsumedOutput>::batch_insert(storage, &mut batch, &output_id, &consumed_output).unwrap();
        consumed_outputs.insert(output_id, consumed_output);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((output_id, consumed_output)) = stream.next().await {
        assert_eq!(consumed_outputs.get(&output_id).unwrap(), &consumed_output);
        count += 1;
    }

    assert_eq!(count, consumed_outputs.len());

    Truncate::<OutputId, ConsumedOutput>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
