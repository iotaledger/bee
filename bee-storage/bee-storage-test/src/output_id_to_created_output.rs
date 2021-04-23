// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::CreatedOutput;
use bee_message::output::OutputId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::output::{rand_created_output, rand_output_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, CreatedOutput>
    + Fetch<OutputId, CreatedOutput>
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

    assert!(
        !Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .is_none()
    );

    Insert::<OutputId, CreatedOutput>::insert(storage, &output_id, &created_output)
        .await
        .unwrap();

    assert!(
        Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .unwrap(),
        created_output
    );

    Delete::<OutputId, CreatedOutput>::delete(storage, &output_id)
        .await
        .unwrap();

    assert!(
        !Exist::<OutputId, CreatedOutput>::exist(storage, &output_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<OutputId, CreatedOutput>::fetch(storage, &output_id)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (output_id, created_output) = (rand_output_id(), rand_created_output());
        Insert::<OutputId, CreatedOutput>::insert(storage, &output_id, &created_output)
            .await
            .unwrap();
        Batch::<OutputId, CreatedOutput>::batch_delete(storage, &mut batch, &output_id).unwrap();
    }

    let mut created_outputs = HashMap::new();

    for _ in 0usize..10usize {
        let (output_id, created_output) = (rand_output_id(), rand_created_output());
        Batch::<OutputId, CreatedOutput>::batch_insert(storage, &mut batch, &output_id, &created_output).unwrap();
        created_outputs.insert(output_id, created_output);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<OutputId, CreatedOutput>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((output_id, created_output)) = stream.next().await {
        assert_eq!(created_outputs.get(&output_id).unwrap(), &created_output);
        count += 1;
    }

    assert_eq!(count, created_outputs.len());

    Truncate::<OutputId, CreatedOutput>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<OutputId, CreatedOutput>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
