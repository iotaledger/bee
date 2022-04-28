// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_tangle::milestone_metadata::MilestoneMetadata;
use bee_test::rand::milestone::{rand_milestone_index, rand_milestone_metadata};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneIndex, MilestoneMetadata>
    + Fetch<MilestoneIndex, MilestoneMetadata>
    + for<'a> MultiFetch<'a, MilestoneIndex, MilestoneMetadata>
    + Insert<MilestoneIndex, MilestoneMetadata>
    + Delete<MilestoneIndex, MilestoneMetadata>
    + BatchBuilder
    + Batch<MilestoneIndex, MilestoneMetadata>
    + for<'a> AsIterator<'a, MilestoneIndex, MilestoneMetadata>
    + Truncate<MilestoneIndex, MilestoneMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneIndex, MilestoneMetadata>
        + Fetch<MilestoneIndex, MilestoneMetadata>
        + for<'a> MultiFetch<'a, MilestoneIndex, MilestoneMetadata>
        + Insert<MilestoneIndex, MilestoneMetadata>
        + Delete<MilestoneIndex, MilestoneMetadata>
        + BatchBuilder
        + Batch<MilestoneIndex, MilestoneMetadata>
        + for<'a> AsIterator<'a, MilestoneIndex, MilestoneMetadata>
        + Truncate<MilestoneIndex, MilestoneMetadata>
{
}

pub fn milestone_index_to_milestone_metadata_access<B: StorageBackend>(storage: &B) {
    let (index, milestone) = (rand_milestone_index(), rand_milestone_metadata());

    assert!(!Exist::<MilestoneIndex, MilestoneMetadata>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, MilestoneMetadata>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MilestoneIndex, MilestoneMetadata>::insert(storage, &index, &milestone).unwrap();

    assert!(Exist::<MilestoneIndex, MilestoneMetadata>::exist(storage, &index).unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(storage, &index)
            .unwrap()
            .unwrap(),
        milestone
    );
    let results = MultiFetch::<MilestoneIndex, MilestoneMetadata>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &milestone));

    Delete::<MilestoneIndex, MilestoneMetadata>::delete(storage, &index).unwrap();

    assert!(!Exist::<MilestoneIndex, MilestoneMetadata>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, MilestoneMetadata>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, MilestoneMetadata>::multi_fetch(storage, &[index])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut indexes = Vec::new();
    let mut milestones = Vec::new();

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone_metadata());
        Insert::<MilestoneIndex, MilestoneMetadata>::insert(storage, &index, &milestone).unwrap();
        Batch::<MilestoneIndex, MilestoneMetadata>::batch_delete(storage, &mut batch, &index).unwrap();
        indexes.push(index);
        milestones.push((index, None));
    }

    for _ in 0..10 {
        let (index, milestone) = (rand_milestone_index(), rand_milestone_metadata());
        Batch::<MilestoneIndex, MilestoneMetadata>::batch_insert(storage, &mut batch, &index, &milestone).unwrap();
        indexes.push(index);
        milestones.push((index, Some(milestone)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MilestoneIndex, MilestoneMetadata>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (index, milestone) = result.unwrap();
        assert!(milestones.contains(&(index, Some(milestone))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MilestoneIndex, MilestoneMetadata>::multi_fetch(storage, &indexes)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), indexes.len());

    for ((_, milestone), result) in milestones.into_iter().zip(results.into_iter()) {
        assert_eq!(milestone, result.unwrap());
    }

    Truncate::<MilestoneIndex, MilestoneMetadata>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MilestoneIndex, MilestoneMetadata>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
