// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_block::payload::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::unreferenced_block::UnreferencedBlock;
use bee_test::rand::{milestone::rand_milestone_index, unreferenced_block::rand_unreferenced_block};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MilestoneIndex, UnreferencedBlock), ()>
    + Fetch<MilestoneIndex, Vec<UnreferencedBlock>>
    + Insert<(MilestoneIndex, UnreferencedBlock), ()>
    + Delete<(MilestoneIndex, UnreferencedBlock), ()>
    + BatchBuilder
    + Batch<(MilestoneIndex, UnreferencedBlock), ()>
    + for<'a> AsIterator<'a, (MilestoneIndex, UnreferencedBlock), ()>
    + Truncate<(MilestoneIndex, UnreferencedBlock), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MilestoneIndex, UnreferencedBlock), ()>
        + Fetch<MilestoneIndex, Vec<UnreferencedBlock>>
        + Insert<(MilestoneIndex, UnreferencedBlock), ()>
        + Delete<(MilestoneIndex, UnreferencedBlock), ()>
        + BatchBuilder
        + Batch<(MilestoneIndex, UnreferencedBlock), ()>
        + for<'a> AsIterator<'a, (MilestoneIndex, UnreferencedBlock), ()>
        + Truncate<(MilestoneIndex, UnreferencedBlock), ()>
{
}

pub fn milestone_index_to_unreferenced_block_access<B: StorageBackend>(storage: &B) {
    let (index, unreferenced_block) = (rand_milestone_index(), rand_unreferenced_block());

    assert!(!Exist::<(MilestoneIndex, UnreferencedBlock), ()>::exist(storage, &(index, unreferenced_block)).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Vec<UnreferencedBlock>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(MilestoneIndex, UnreferencedBlock), ()>::insert(storage, &(index, unreferenced_block), &()).unwrap();

    assert!(Exist::<(MilestoneIndex, UnreferencedBlock), ()>::exist(storage, &(index, unreferenced_block)).unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<UnreferencedBlock>>::fetch(storage, &index)
            .unwrap()
            .unwrap(),
        vec![unreferenced_block]
    );

    Delete::<(MilestoneIndex, UnreferencedBlock), ()>::delete(storage, &(index, unreferenced_block)).unwrap();

    assert!(!Exist::<(MilestoneIndex, UnreferencedBlock), ()>::exist(storage, &(index, unreferenced_block)).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Vec<UnreferencedBlock>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, unreferenced_block) = (rand_milestone_index(), rand_unreferenced_block());
        Insert::<(MilestoneIndex, UnreferencedBlock), ()>::insert(storage, &(index, unreferenced_block), &()).unwrap();
        Batch::<(MilestoneIndex, UnreferencedBlock), ()>::batch_delete(
            storage,
            &mut batch,
            &(index, unreferenced_block),
        )
        .unwrap();
    }

    let mut unreferenced_blocks = HashMap::<MilestoneIndex, Vec<UnreferencedBlock>>::new();

    for _ in 0..5 {
        let index = rand_milestone_index();
        for _ in 0..5 {
            let unreferenced_block = rand_unreferenced_block();
            Batch::<(MilestoneIndex, UnreferencedBlock), ()>::batch_insert(
                storage,
                &mut batch,
                &(index, unreferenced_block),
                &(),
            )
            .unwrap();
            unreferenced_blocks.entry(index).or_default().push(unreferenced_block);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(MilestoneIndex, UnreferencedBlock), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, message_id), _) = result.unwrap();
        assert!(unreferenced_blocks.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, unreferenced_blocks.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, UnreferencedBlock), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(MilestoneIndex, UnreferencedBlock), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
