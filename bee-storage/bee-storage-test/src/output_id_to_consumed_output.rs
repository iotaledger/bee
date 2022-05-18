// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::ConsumedOutput;
use bee_message::output::OutputId;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::output::{rand_consumed_output, rand_output_id};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<OutputId, ConsumedOutput>
    + Fetch<OutputId, ConsumedOutput>
    + for<'a> MultiFetch<'a, OutputId, ConsumedOutput>
    + Insert<OutputId, ConsumedOutput>
    + Delete<OutputId, ConsumedOutput>
    + BatchBuilder
    + Batch<OutputId, ConsumedOutput>
    + for<'a> AsIterator<'a, OutputId, ConsumedOutput>
    + Truncate<OutputId, ConsumedOutput>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<OutputId, ConsumedOutput>
        + Fetch<OutputId, ConsumedOutput>
        + for<'a> MultiFetch<'a, OutputId, ConsumedOutput>
        + Insert<OutputId, ConsumedOutput>
        + Delete<OutputId, ConsumedOutput>
        + BatchBuilder
        + Batch<OutputId, ConsumedOutput>
        + for<'a> AsIterator<'a, OutputId, ConsumedOutput>
        + Truncate<OutputId, ConsumedOutput>
{
}

pub fn output_id_to_consumed_output_access<B: StorageBackend>(storage: &B) {
    let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());

    assert!(!storage.exist::<OutputId, ConsumedOutput>(&output_id).unwrap());
    assert!(storage.fetch::<OutputId, ConsumedOutput>(&output_id).unwrap().is_none());
    let results = storage
        .multi_fetch::<OutputId, ConsumedOutput>(&[output_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    storage
        .insert::<OutputId, ConsumedOutput>(&output_id, &consumed_output)
        .unwrap();

    assert!(storage.exist::<OutputId, ConsumedOutput>(&output_id).unwrap());
    assert_eq!(
        storage.fetch::<OutputId, ConsumedOutput>(&output_id).unwrap().unwrap(),
        consumed_output
    );
    let results = storage
        .multi_fetch::<OutputId, ConsumedOutput>(&[output_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &consumed_output));

    storage.delete::<OutputId, ConsumedOutput>(&output_id).unwrap();

    assert!(!storage.exist::<OutputId, ConsumedOutput>(&output_id).unwrap());
    assert!(storage.fetch::<OutputId, ConsumedOutput>(&output_id).unwrap().is_none());
    let results = storage
        .multi_fetch::<OutputId, ConsumedOutput>(&[output_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut output_ids = Vec::new();
    let mut consumed_outputs = Vec::new();

    for _ in 0..10 {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        storage
            .insert::<OutputId, ConsumedOutput>(&output_id, &consumed_output)
            .unwrap();
        storage
            .batch_delete::<OutputId, ConsumedOutput>(&mut batch, &output_id)
            .unwrap();
        output_ids.push(output_id);
        consumed_outputs.push((output_id, None));
    }

    for _ in 0..10 {
        let (output_id, consumed_output) = (rand_output_id(), rand_consumed_output());
        storage
            .batch_insert::<OutputId, ConsumedOutput>(&mut batch, &output_id, &consumed_output)
            .unwrap();
        output_ids.push(output_id);
        consumed_outputs.push((output_id, Some(consumed_output)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<OutputId, ConsumedOutput>().unwrap();
    let mut count = 0;

    for result in iter {
        let (output_id, consumed_output) = result.unwrap();
        assert!(consumed_outputs.contains(&(output_id, Some(consumed_output))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = storage
        .multi_fetch::<OutputId, ConsumedOutput>(&output_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), output_ids.len());

    for ((_, consumed_output), result) in consumed_outputs.into_iter().zip(results.into_iter()) {
        assert_eq!(consumed_output, result.unwrap());
    }

    storage.truncate::<OutputId, ConsumedOutput>().unwrap();

    let mut iter = storage.iter::<OutputId, ConsumedOutput>().unwrap();

    assert!(iter.next().is_none());
}
