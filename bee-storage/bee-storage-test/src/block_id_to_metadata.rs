// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{payload::milestone::MilestoneIndex, rand::block::rand_block_id, BlockId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, InsertStrict, MultiFetch, Truncate, Update},
    backend,
};
use bee_tangle::{block_metadata::BlockMetadata, rand::block_metadata::rand_block_metadata};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<BlockId, BlockMetadata>
    + Fetch<BlockId, BlockMetadata>
    + for<'a> MultiFetch<'a, BlockId, BlockMetadata>
    + InsertStrict<BlockId, BlockMetadata>
    + Delete<BlockId, BlockMetadata>
    + BatchBuilder
    + Batch<BlockId, BlockMetadata>
    + for<'a> AsIterator<'a, BlockId, BlockMetadata>
    + Truncate<BlockId, BlockMetadata>
    + Update<BlockId, BlockMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<BlockId, BlockMetadata>
        + Fetch<BlockId, BlockMetadata>
        + for<'a> MultiFetch<'a, BlockId, BlockMetadata>
        + InsertStrict<BlockId, BlockMetadata>
        + Delete<BlockId, BlockMetadata>
        + BatchBuilder
        + Batch<BlockId, BlockMetadata>
        + for<'a> AsIterator<'a, BlockId, BlockMetadata>
        + Truncate<BlockId, BlockMetadata>
        + Update<BlockId, BlockMetadata>
{
}

pub fn block_id_to_metadata_access<B: StorageBackend>(storage: &B) {
    let (block_id, metadata) = (rand_block_id(), rand_block_metadata());

    assert!(!Exist::<BlockId, BlockMetadata>::exist(storage, &block_id).unwrap());
    assert!(
        Fetch::<BlockId, BlockMetadata>::fetch(storage, &block_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<BlockId, BlockMetadata>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    InsertStrict::<BlockId, BlockMetadata>::insert_strict(storage, &block_id, &metadata).unwrap();
    assert!(Exist::<BlockId, BlockMetadata>::exist(storage, &block_id).unwrap());

    // calling `insert_strict` with the same `BlockId` but a different `BlockMetadata` should
    // not overwrite the old value.
    {
        let index = metadata.milestone_index().map_or(0, |i| *i + 1);
        let mut metadata = metadata;
        metadata.set_milestone_index(MilestoneIndex(index));

        InsertStrict::<BlockId, BlockMetadata>::insert_strict(storage, &block_id, &metadata).unwrap();
    }
    assert_eq!(
        Fetch::<BlockId, BlockMetadata>::fetch(storage, &block_id)
            .unwrap()
            .unwrap(),
        metadata,
        "`InsertStrict` should not overwrite"
    );

    let results = MultiFetch::<BlockId, BlockMetadata>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &metadata));

    let milestone_index = {
        let index = Fetch::<BlockId, BlockMetadata>::fetch(storage, &block_id)
            .unwrap()
            .unwrap()
            .milestone_index();

        MilestoneIndex(index.map_or(0, |i| i.wrapping_add(1)))
    };

    Update::<BlockId, BlockMetadata>::update(storage, &block_id, |metadata: &mut BlockMetadata| {
        metadata.set_milestone_index(milestone_index);
    })
    .unwrap();

    assert_eq!(
        Fetch::<BlockId, BlockMetadata>::fetch(storage, &block_id)
            .unwrap()
            .unwrap()
            .milestone_index(),
        Some(milestone_index),
    );

    Delete::<BlockId, BlockMetadata>::delete(storage, &block_id).unwrap();

    assert!(!Exist::<BlockId, BlockMetadata>::exist(storage, &block_id).unwrap());
    assert!(
        Fetch::<BlockId, BlockMetadata>::fetch(storage, &block_id)
            .unwrap()
            .is_none()
    );

    let results = MultiFetch::<BlockId, BlockMetadata>::multi_fetch(storage, &[block_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut block_ids = Vec::new();
    let mut metadatas = Vec::new();

    for _ in 0..10 {
        let (block_id, metadata) = (rand_block_id(), rand_block_metadata());
        InsertStrict::<BlockId, BlockMetadata>::insert_strict(storage, &block_id, &metadata).unwrap();
        Batch::<BlockId, BlockMetadata>::batch_delete(storage, &mut batch, &block_id).unwrap();
        block_ids.push(block_id);
        metadatas.push((block_id, None));
    }

    for _ in 0..10 {
        let (block_id, metadata) = (rand_block_id(), rand_block_metadata());
        Batch::<BlockId, BlockMetadata>::batch_insert(storage, &mut batch, &block_id, &metadata).unwrap();
        block_ids.push(block_id);
        metadatas.push((block_id, Some(metadata)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<BlockId, BlockMetadata>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (block_id, metadata) = result.unwrap();
        assert!(metadatas.contains(&(block_id, Some(metadata))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<BlockId, BlockMetadata>::multi_fetch(storage, &block_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), block_ids.len());

    for ((_, metadata), result) in metadatas.into_iter().zip(results.into_iter()) {
        assert_eq!(metadata, result.unwrap());
    }

    Truncate::<BlockId, BlockMetadata>::truncate(storage).unwrap();

    let mut iter = AsIterator::<BlockId, BlockMetadata>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
