// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::milestone::{rand_milestone, rand_milestone_index};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneIndex, Milestone>
    + Fetch<MilestoneIndex, Milestone>
    + for<'a> MultiFetch<'a, MilestoneIndex, Milestone>
    + Insert<MilestoneIndex, Milestone>
    + Delete<MilestoneIndex, Milestone>
    + BatchBuilder
    + Batch<MilestoneIndex, Milestone>
    + for<'a> AsIterator<'a, MilestoneIndex, Milestone>
    + Truncate<MilestoneIndex, Milestone>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneIndex, Milestone>
        + Fetch<MilestoneIndex, Milestone>
        + for<'a> MultiFetch<'a, MilestoneIndex, Milestone>
        + Insert<MilestoneIndex, Milestone>
        + Delete<MilestoneIndex, Milestone>
        + BatchBuilder
        + Batch<MilestoneIndex, Milestone>
        + for<'a> AsIterator<'a, MilestoneIndex, Milestone>
        + Truncate<MilestoneIndex, Milestone>
{
}

pub fn milestone_index_to_milestone_access<B: StorageBackend>(storage: &B) {
    let (index, milestone) = (rand_milestone_index(), rand_milestone());

    assert!(!Exist::<MilestoneIndex, Milestone>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, Milestone>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MilestoneIndex, Milestone>::insert(storage, &index, &milestone).unwrap();

    assert!(Exist::<MilestoneIndex, Milestone>::exist(storage, &index).unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .unwrap()
            .unwrap(),
        milestone
    );
    let results = MultiFetch::<MilestoneIndex, Milestone>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &milestone));

    Delete::<MilestoneIndex, Milestone>::delete(storage, &index).unwrap();

    assert!(!Exist::<MilestoneIndex, Milestone>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, Milestone>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut indexes = Vec::new();
    let mut milestones = Vec::new();

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        Insert::<MilestoneIndex, Milestone>::insert(storage, &index, &milestone).unwrap();
        Batch::<MilestoneIndex, Milestone>::batch_delete(storage, &mut batch, &index).unwrap();
        indexes.push(index);
        milestones.push((index, None));
    }

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        Batch::<MilestoneIndex, Milestone>::batch_insert(storage, &mut batch, &index, &milestone).unwrap();
        indexes.push(index);
        milestones.push((index, Some(milestone)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MilestoneIndex, Milestone>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (index, milestone) = result.unwrap();
        assert!(milestones.contains(&(index, Some(milestone))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MilestoneIndex, Milestone>::multi_fetch(storage, &indexes)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), indexes.len());

    for ((_, milestone), result) in milestones.into_iter().zip(results.into_iter()) {
        assert_eq!(milestone, result.unwrap());
    }

    Truncate::<MilestoneIndex, Milestone>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MilestoneIndex, Milestone>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
