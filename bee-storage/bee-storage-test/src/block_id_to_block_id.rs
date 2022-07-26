// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_block::{rand::block::rand_block_id, BlockId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(BlockId, BlockId), ()>
    + Fetch<BlockId, Vec<BlockId>>
    + Insert<(BlockId, BlockId), ()>
    + Delete<(BlockId, BlockId), ()>
    + BatchBuilder
    + Batch<(BlockId, BlockId), ()>
    + for<'a> AsIterator<'a, (BlockId, BlockId), ()>
    + Truncate<(BlockId, BlockId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(BlockId, BlockId), ()>
        + Fetch<BlockId, Vec<BlockId>>
        + Insert<(BlockId, BlockId), ()>
        + Delete<(BlockId, BlockId), ()>
        + BatchBuilder
        + Batch<(BlockId, BlockId), ()>
        + for<'a> AsIterator<'a, (BlockId, BlockId), ()>
        + Truncate<(BlockId, BlockId), ()>
{
}

pub fn block_id_to_block_id_access<B: StorageBackend>(storage: &B) {
    let (parent, child) = (rand_block_id(), rand_block_id());

    assert!(!Exist::<(BlockId, BlockId), ()>::exist(storage, &(parent, child)).unwrap());
    assert!(
        Fetch::<BlockId, Vec<BlockId>>::fetch(storage, &parent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(BlockId, BlockId), ()>::insert(storage, &(parent, child), &()).unwrap();

    assert!(Exist::<(BlockId, BlockId), ()>::exist(storage, &(parent, child)).unwrap());
    assert_eq!(
        Fetch::<BlockId, Vec<BlockId>>::fetch(storage, &parent)
            .unwrap()
            .unwrap(),
        vec![child]
    );

    Delete::<(BlockId, BlockId), ()>::delete(storage, &(parent, child)).unwrap();

    assert!(!Exist::<(BlockId, BlockId), ()>::exist(storage, &(parent, child)).unwrap());
    assert!(
        Fetch::<BlockId, Vec<BlockId>>::fetch(storage, &parent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (parent, child) = (rand_block_id(), rand_block_id());
        Insert::<(BlockId, BlockId), ()>::insert(storage, &(parent, child), &()).unwrap();
        Batch::<(BlockId, BlockId), ()>::batch_delete(storage, &mut batch, &(parent, child)).unwrap();
    }

    let mut edges = HashMap::<BlockId, Vec<BlockId>>::new();

    for _ in 0..5 {
        let parent = rand_block_id();
        for _ in 0..5 {
            let child = rand_block_id();
            Batch::<(BlockId, BlockId), ()>::batch_insert(storage, &mut batch, &(parent, child), &()).unwrap();
            edges.entry(parent).or_default().push(child);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(BlockId, BlockId), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((parent, child), _) = result.unwrap();
        assert!(edges.get(&parent).unwrap().contains(&child));
        count += 1;
    }

    assert_eq!(count, edges.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(BlockId, BlockId), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(BlockId, BlockId), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
