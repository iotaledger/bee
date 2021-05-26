// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::ConsumedOutput;
use bee_message::output::OutputId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::output::{rand_consumed_output, rand_output_id};

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, ConsumedOutput>
    + Fetch<OutputId, ConsumedOutput>
    + MultiFetch<OutputId, ConsumedOutput>
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
        + MultiFetch<OutputId, ConsumedOutput>
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

    assert!(
        !Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<OutputId, ConsumedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_none());

    Insert::<OutputId, ConsumedOutput>::insert(storage, &output_id, &consumed_output)
        .await
        .unwrap();

    assert!(
        Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .unwrap(),
        consumed_output
    );
    let results = MultiFetch::<OutputId, ConsumedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_ref().unwrap(), &consumed_output);

    Delete::<OutputId, ConsumedOutput>::delete(storage, &output_id)
        .await
        .unwrap();

    assert!(
        !Exist::<OutputId, ConsumedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<OutputId, ConsumedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<OutputId, ConsumedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_none());

    let mut batch = B::batch_begin();
    let mut output_ids = Vec::new();
    let mut consumed_outputs = Vec::new();

    for _ in 0..10 {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        Insert::<OutputId, ConsumedOutput>::insert(storage, &output_id, &consumed_output)
            .await
            .unwrap();
        Batch::<OutputId, ConsumedOutput>::batch_delete(storage, &mut batch, &output_id).unwrap();
        output_ids.push(output_id);
        consumed_outputs.push((output_id, None));
    }

    for _ in 0..10 {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        Batch::<OutputId, ConsumedOutput>::batch_insert(storage, &mut batch, &output_id, &consumed_output).unwrap();
        output_ids.push(output_id);
        consumed_outputs.push((output_id, Some(consumed_output)));
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (output_id, consumed_output) = result.unwrap();
        assert!(consumed_outputs.contains(&(output_id, Some(consumed_output))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<OutputId, ConsumedOutput>::multi_fetch(storage, &output_ids)
        .await
        .unwrap();

    assert_eq!(results.len(), output_ids.len());

    for ((_, consumed_output), result) in consumed_outputs.iter().zip(results.iter()) {
        assert_eq!(consumed_output, result);
    }

    Truncate::<OutputId, ConsumedOutput>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<OutputId, ConsumedOutput>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
