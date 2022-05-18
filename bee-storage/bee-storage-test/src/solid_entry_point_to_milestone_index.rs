// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_tangle::solid_entry_point::SolidEntryPoint;
use bee_test::rand::{milestone::rand_milestone_index, solid_entry_point::rand_solid_entry_point};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<SolidEntryPoint, MilestoneIndex>
    + Fetch<SolidEntryPoint, MilestoneIndex>
    + for<'a> MultiFetch<'a, SolidEntryPoint, MilestoneIndex>
    + Insert<SolidEntryPoint, MilestoneIndex>
    + Delete<SolidEntryPoint, MilestoneIndex>
    + BatchBuilder
    + Batch<SolidEntryPoint, MilestoneIndex>
    + for<'a> AsIterator<'a, SolidEntryPoint, MilestoneIndex>
    + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<SolidEntryPoint, MilestoneIndex>
        + Fetch<SolidEntryPoint, MilestoneIndex>
        + for<'a> MultiFetch<'a, SolidEntryPoint, MilestoneIndex>
        + Insert<SolidEntryPoint, MilestoneIndex>
        + Delete<SolidEntryPoint, MilestoneIndex>
        + BatchBuilder
        + Batch<SolidEntryPoint, MilestoneIndex>
        + for<'a> AsIterator<'a, SolidEntryPoint, MilestoneIndex>
        + Truncate<SolidEntryPoint, MilestoneIndex>
{
}

pub fn solid_entry_point_to_milestone_index_access<B: StorageBackend>(storage: &B) {
    let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());

    assert!(!Exist::<SolidEntryPoint, MilestoneIndex>::exist_op(storage, &sep).unwrap());
    assert!(
        storage
            .fetch::<SolidEntryPoint, MilestoneIndex>(&sep)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<SolidEntryPoint, MilestoneIndex>::multi_fetch_op(storage, &[sep])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<SolidEntryPoint, MilestoneIndex>::insert_op(storage, &sep, &index).unwrap();

    assert!(Exist::<SolidEntryPoint, MilestoneIndex>::exist_op(storage, &sep).unwrap());
    assert_eq!(
        storage.fetch::<SolidEntryPoint, MilestoneIndex>(&sep).unwrap().unwrap(),
        index
    );
    let results = MultiFetch::<SolidEntryPoint, MilestoneIndex>::multi_fetch_op(storage, &[sep])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &index));

    Delete::<SolidEntryPoint, MilestoneIndex>::delete_op(storage, &sep).unwrap();

    assert!(!Exist::<SolidEntryPoint, MilestoneIndex>::exist_op(storage, &sep).unwrap());
    assert!(
        storage
            .fetch::<SolidEntryPoint, MilestoneIndex>(&sep)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<SolidEntryPoint, MilestoneIndex>::multi_fetch_op(storage, &[sep])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut seps_ids = Vec::new();
    let mut seps = Vec::new();

    for _ in 0..10 {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Insert::<SolidEntryPoint, MilestoneIndex>::insert_op(storage, &sep, &index).unwrap();
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete_op(storage, &mut batch, &sep).unwrap();
        seps_ids.push(sep);
        seps.push((sep, None));
    }

    for _ in 0..10 {
        let (sep, index) = (rand_solid_entry_point(), rand_milestone_index());
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert_op(storage, &mut batch, &sep, &index).unwrap();
        seps_ids.push(sep);
        seps.push((sep, Some(index)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<SolidEntryPoint, MilestoneIndex>::iter_op(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (sep, index) = result.unwrap();
        assert!(seps.contains(&(sep, Some(index))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<SolidEntryPoint, MilestoneIndex>::multi_fetch_op(storage, &seps_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), seps_ids.len());

    for ((_, index), result) in seps.into_iter().zip(results.into_iter()) {
        assert_eq!(index, result.unwrap());
    }

    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate_op(storage).unwrap();

    let mut iter = AsIterator::<SolidEntryPoint, MilestoneIndex>::iter_op(storage).unwrap();

    assert!(iter.next().is_none());
}
