// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{Block, BlockId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::block::{rand_block_id, rand_message};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<BlockId, Block>
    + Fetch<BlockId, Block>
    + for<'a> MultiFetch<'a, BlockId, Block>
    + Insert<BlockId, Block>
    + Delete<BlockId, Block>
    + BatchBuilder
    + Batch<BlockId, Block>
    + for<'a> AsIterator<'a, BlockId, Block>
    + Truncate<BlockId, Block>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<BlockId, Block>
        + Fetch<BlockId, Block>
        + for<'a> MultiFetch<'a, BlockId, Block>
        + Insert<BlockId, Block>
        + Delete<BlockId, Block>
        + BatchBuilder
        + Batch<BlockId, Block>
        + for<'a> AsIterator<'a, BlockId, Block>
        + Truncate<BlockId, Block>
{
}

pub fn message_id_to_message_access<B: StorageBackend>(storage: &B) {
    let (message_id, message) = (rand_block_id(), rand_message());

    assert!(!Exist::<BlockId, Block>::exist(storage, &message_id).unwrap());
    assert!(Fetch::<BlockId, Block>::fetch(storage, &message_id).unwrap().is_none());
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<BlockId, Block>::insert(storage, &message_id, &message).unwrap();

    let message = rand_message();
    Insert::<BlockId, Block>::insert(storage, &message_id, &message).unwrap();
    assert_eq!(
        Fetch::<BlockId, Block>::fetch(storage, &message_id).unwrap().as_ref(),
        Some(&message),
        "insert should overwrite"
    );

    assert!(Exist::<BlockId, Block>::exist(storage, &message_id).unwrap());
    assert_eq!(
        Fetch::<BlockId, Block>::fetch(storage, &message_id).unwrap().unwrap(),
        message
    );
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &message));

    Delete::<BlockId, Block>::delete(storage, &message_id).unwrap();

    assert!(!Exist::<BlockId, Block>::exist(storage, &message_id).unwrap());
    assert!(Fetch::<BlockId, Block>::fetch(storage, &message_id).unwrap().is_none());
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut message_ids = Vec::new();
    let mut messages = Vec::new();

    for _ in 0..10 {
        let (message_id, message) = (rand_block_id(), rand_message());
        Insert::<BlockId, Block>::insert(storage, &message_id, &message).unwrap();
        Batch::<BlockId, Block>::batch_delete(storage, &mut batch, &message_id).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, message) = (rand_block_id(), rand_message());
        Batch::<BlockId, Block>::batch_insert(storage, &mut batch, &message_id, &message).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, Some(message)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<BlockId, Block>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (message_id, message) = result.unwrap();
        assert!(messages.contains(&(message_id, Some(message))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &message_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), message_ids.len());

    for ((_, message), result) in messages.into_iter().zip(results.into_iter()) {
        assert_eq!(message, result.unwrap());
    }

    Truncate::<BlockId, Block>::truncate(storage).unwrap();

    let mut iter = AsIterator::<BlockId, Block>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
