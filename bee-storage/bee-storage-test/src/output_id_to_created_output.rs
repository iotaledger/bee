// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::CreatedOutput;
use bee_message::output::OutputId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::output::{rand_created_output, rand_output_id};

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, CreatedOutput>
    + Fetch<OutputId, CreatedOutput>
    + MultiFetch<OutputId, CreatedOutput>
    + Insert<OutputId, CreatedOutput>
    + Delete<OutputId, CreatedOutput>
    + BatchBuilder
    + Batch<OutputId, CreatedOutput>
    + for<'a> AsStream<'a, OutputId, CreatedOutput>
    + Truncate<OutputId, CreatedOutput>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<OutputId, CreatedOutput>
        + Fetch<OutputId, CreatedOutput>
        + MultiFetch<OutputId, CreatedOutput>
        + Insert<OutputId, CreatedOutput>
        + Delete<OutputId, CreatedOutput>
        + BatchBuilder
        + Batch<OutputId, CreatedOutput>
        + for<'a> AsStream<'a, OutputId, CreatedOutput>
        + Truncate<OutputId, CreatedOutput>
{
}

pub async fn output_id_to_created_output_access<B: StorageBackend>(storage: &B) {
    let (output_id, created_output) = (rand_output_id(), rand_created_output());

    assert!(!Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert!(Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
        .await
        .unwrap()
        .is_none());
    let results = MultiFetch::<OutputId, CreatedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<OutputId, CreatedOutput>::insert(storage, &output_id, &created_output)
        .await
        .unwrap();

    assert!(Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert_eq!(
        Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .unwrap(),
        created_output
    );
    let results = MultiFetch::<OutputId, CreatedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &created_output));

    Delete::<OutputId, CreatedOutput>::delete(storage, &output_id)
        .await
        .unwrap();

    assert!(!Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
        .await
        .unwrap());
    assert!(Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
        .await
        .unwrap()
        .is_none());
    let results = MultiFetch::<OutputId, CreatedOutput>::multi_fetch(storage, &[output_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut output_ids = Vec::new();
    let mut created_outputs = Vec::new();

    for _ in 0..10 {
        let (output_id, created_output) = (rand_output_id(), rand_created_output());
        Insert::<OutputId, CreatedOutput>::insert(storage, &output_id, &created_output)
            .await
            .unwrap();
        Batch::<OutputId, CreatedOutput>::batch_delete(storage, &mut batch, &output_id).unwrap();
        output_ids.push(output_id);
        created_outputs.push((output_id, None));
    }

    for _ in 0..10 {
        let (output_id, created_output) = (rand_output_id(), rand_created_output());
        Batch::<OutputId, CreatedOutput>::batch_insert(storage, &mut batch, &output_id, &created_output).unwrap();
        output_ids.push(output_id);
        created_outputs.push((output_id, Some(created_output)));
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<OutputId, CreatedOutput>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (output_id, created_output) = result.unwrap();
        assert!(created_outputs.contains(&(output_id, Some(created_output))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<OutputId, CreatedOutput>::multi_fetch(storage, &output_ids)
        .await
        .unwrap();

    assert_eq!(results.len(), output_ids.len());

    for ((_, created_output), result) in created_outputs.into_iter().zip(results.into_iter()) {
        assert_eq!(created_output, result.unwrap());
    }

    Truncate::<OutputId, CreatedOutput>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<OutputId, CreatedOutput>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
