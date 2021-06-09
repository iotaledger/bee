// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::OutputDiff;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::{milestone::rand_milestone_index, output_diff::rand_output_diff};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneIndex, OutputDiff>
    + Fetch<MilestoneIndex, OutputDiff>
    + MultiFetch<MilestoneIndex, OutputDiff>
    + Insert<MilestoneIndex, OutputDiff>
    + Delete<MilestoneIndex, OutputDiff>
    + BatchBuilder
    + Batch<MilestoneIndex, OutputDiff>
    + for<'a> AsIterator<'a, MilestoneIndex, OutputDiff>
    + Truncate<MilestoneIndex, OutputDiff>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneIndex, OutputDiff>
        + Fetch<MilestoneIndex, OutputDiff>
        + MultiFetch<MilestoneIndex, OutputDiff>
        + Insert<MilestoneIndex, OutputDiff>
        + Delete<MilestoneIndex, OutputDiff>
        + BatchBuilder
        + Batch<MilestoneIndex, OutputDiff>
        + for<'a> AsIterator<'a, MilestoneIndex, OutputDiff>
        + Truncate<MilestoneIndex, OutputDiff>
{
}

pub fn milestone_index_to_output_diff_access<B: StorageBackend>(storage: &B) {
    let (index, output_diff) = (rand_milestone_index(), rand_output_diff());

    assert!(!Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, OutputDiff>::multi_fetch(storage, &[index]).unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MilestoneIndex, OutputDiff>::insert(storage, &index, &output_diff).unwrap();

    assert!(Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index).unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .pack_new(),
        output_diff.pack_new()
    );
    let results = MultiFetch::<MilestoneIndex, OutputDiff>::multi_fetch(storage, &[index]).unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &output_diff));

    Delete::<MilestoneIndex, OutputDiff>::delete(storage, &index).unwrap();

    assert!(!Exist::<MilestoneIndex, OutputDiff>::exist(storage, &index).unwrap());
    assert!(
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneIndex, OutputDiff>::multi_fetch(storage, &[index]).unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut indexes = Vec::new();
    let mut output_diffs = Vec::new();

    for _ in 0..10 {
        let (index, output_diff) = (rand_milestone_index(), rand_output_diff());
        Insert::<MilestoneIndex, OutputDiff>::insert(storage, &index, &output_diff).unwrap();
        Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, &mut batch, &index).unwrap();
        indexes.push(index);
        output_diffs.push((index, None));
    }

    for _ in 0..10 {
        let (index, output_diff) = (rand_milestone_index(), rand_output_diff());
        Batch::<MilestoneIndex, OutputDiff>::batch_insert(storage, &mut batch, &index, &output_diff).unwrap();
        indexes.push(index);
        output_diffs.push((index, Some(output_diff)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MilestoneIndex, OutputDiff>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (index, output_diff) = result.unwrap();
        assert!(output_diffs.contains(&(index, Some(output_diff))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MilestoneIndex, OutputDiff>::multi_fetch(storage, &indexes).unwrap();

    assert_eq!(results.len(), indexes.len());

    for ((_, diff), result) in output_diffs.into_iter().zip(results.into_iter()) {
        assert_eq!(diff, result.unwrap());
    }

    Truncate::<MilestoneIndex, OutputDiff>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MilestoneIndex, OutputDiff>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
