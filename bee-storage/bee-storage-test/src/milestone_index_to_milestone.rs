// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::{Milestone, MilestoneIndex};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
    backend::StorageBackendExt,
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

    assert!(!storage.exist::<MilestoneIndex, Milestone>(&index).unwrap());
    assert!(storage.fetch::<MilestoneIndex, Milestone>(&index).unwrap().is_none());
    let results = storage
        .multi_fetch::<MilestoneIndex, Milestone>(&[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    storage.insert::<MilestoneIndex, Milestone>(&index, &milestone).unwrap();

    assert!(storage.exist::<MilestoneIndex, Milestone>(&index).unwrap());
    assert_eq!(
        storage.fetch::<MilestoneIndex, Milestone>(&index).unwrap().unwrap(),
        milestone
    );
    let results = storage
        .multi_fetch::<MilestoneIndex, Milestone>(&[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &milestone));

    storage.delete::<MilestoneIndex, Milestone>(&index).unwrap();

    assert!(!storage.exist::<MilestoneIndex, Milestone>(&index).unwrap());
    assert!(storage.fetch::<MilestoneIndex, Milestone>(&index).unwrap().is_none());
    let results = storage
        .multi_fetch::<MilestoneIndex, Milestone>(&[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut indexes = Vec::new();
    let mut milestones = Vec::new();

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        storage.insert::<MilestoneIndex, Milestone>(&index, &milestone).unwrap();
        storage
            .batch_delete::<MilestoneIndex, Milestone>(&mut batch, &index)
            .unwrap();
        indexes.push(index);
        milestones.push((index, None));
    }

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone());
        storage
            .batch_insert::<MilestoneIndex, Milestone>(&mut batch, &index, &milestone)
            .unwrap();
        indexes.push(index);
        milestones.push((index, Some(milestone)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<MilestoneIndex, Milestone>().unwrap();
    let mut count = 0;

    for result in iter {
        let (index, milestone) = result.unwrap();
        assert!(milestones.contains(&(index, Some(milestone))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = storage
        .multi_fetch::<MilestoneIndex, Milestone>(&indexes)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), indexes.len());

    for ((_, milestone), result) in milestones.into_iter().zip(results.into_iter()) {
        assert_eq!(milestone, result.unwrap());
    }

    storage.truncate::<MilestoneIndex, Milestone>().unwrap();

    let mut iter = storage.iter::<MilestoneIndex, Milestone>().unwrap();

    assert!(iter.next().is_none());
}
