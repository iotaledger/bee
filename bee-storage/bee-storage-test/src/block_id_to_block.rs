// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    protocol::protocol_parameters,
    rand::block::{rand_block, rand_block_id},
    Block, BlockId,
};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};

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

pub fn block_id_to_block_access<B: StorageBackend>(storage: &B) {
    let protocol_parameters = protocol_parameters();
    let (block_id, block) = (rand_block_id(), rand_block(&protocol_parameters));

    assert!(!Exist::<BlockId, Block>::exist(storage, &block_id).unwrap());
    assert!(Fetch::<BlockId, Block>::fetch(storage, &block_id).unwrap().is_none());
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<BlockId, Block>::insert(storage, &block_id, &block).unwrap();

    let block = rand_block(&protocol_parameters);
    Insert::<BlockId, Block>::insert(storage, &block_id, &block).unwrap();
    assert_eq!(
        Fetch::<BlockId, Block>::fetch(storage, &block_id).unwrap().as_ref(),
        Some(&block),
        "insert should overwrite"
    );

    assert!(Exist::<BlockId, Block>::exist(storage, &block_id).unwrap());
    assert_eq!(
        Fetch::<BlockId, Block>::fetch(storage, &block_id).unwrap().unwrap(),
        block
    );
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &block));

    Delete::<BlockId, Block>::delete(storage, &block_id).unwrap();

    assert!(!Exist::<BlockId, Block>::exist(storage, &block_id).unwrap());
    assert!(Fetch::<BlockId, Block>::fetch(storage, &block_id).unwrap().is_none());
    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut block_ids = Vec::new();
    let mut blocks = Vec::new();

    for _ in 0..10 {
        let (block_id, block) = (rand_block_id(), rand_block(&protocol_parameters));
        Insert::<BlockId, Block>::insert(storage, &block_id, &block).unwrap();
        Batch::<BlockId, Block>::batch_delete(storage, &mut batch, &block_id).unwrap();
        block_ids.push(block_id);
        blocks.push((block_id, None));
    }

    for _ in 0..10 {
        let (block_id, block) = (rand_block_id(), rand_block(&protocol_parameters));
        Batch::<BlockId, Block>::batch_insert(storage, &mut batch, &block_id, &block).unwrap();
        block_ids.push(block_id);
        blocks.push((block_id, Some(block)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<BlockId, Block>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (block_id, block) = result.unwrap();
        assert!(blocks.contains(&(block_id, Some(block))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<BlockId, Block>::multi_fetch(storage, &block_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), block_ids.len());

    for ((_, block), result) in blocks.into_iter().zip(results.into_iter()) {
        assert_eq!(block, result.unwrap());
    }

    Truncate::<BlockId, Block>::truncate(storage).unwrap();

    let mut iter = AsIterator::<BlockId, Block>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
