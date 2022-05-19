// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::MilestoneIndex;
use bee_ledger::types::LedgerIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(), LedgerIndex>
    + Fetch<(), LedgerIndex>
    + Insert<(), LedgerIndex>
    + Delete<(), LedgerIndex>
    + BatchBuilder
    + Batch<(), LedgerIndex>
    + for<'a> AsIterator<'a, (), LedgerIndex>
    + Truncate<(), LedgerIndex>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(), LedgerIndex>
        + Fetch<(), LedgerIndex>
        + Insert<(), LedgerIndex>
        + Delete<(), LedgerIndex>
        + BatchBuilder
        + Batch<(), LedgerIndex>
        + for<'a> AsIterator<'a, (), LedgerIndex>
        + Truncate<(), LedgerIndex>
{
}

pub fn ledger_index_access<B: StorageBackend>(storage: &B) {
    let index = LedgerIndex::from(MilestoneIndex::from(42));

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(storage, &(), &index).unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());
    assert_eq!(Fetch::<(), LedgerIndex>::fetch(storage, &()).unwrap().unwrap(), index);

    Delete::<(), LedgerIndex>::delete(storage, &()).unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).unwrap().is_none());

    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &index).unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());
    assert_eq!(Fetch::<(), LedgerIndex>::fetch(storage, &()).unwrap().unwrap(), index);

    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_delete(storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(storage, &(), &index).unwrap();

    let iter = AsIterator::<(), LedgerIndex>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (_, ledger_index) = result.unwrap();
        assert_eq!(ledger_index, index);
        count += 1;
    }

    assert_eq!(count, 1);

    Truncate::<(), LedgerIndex>::truncate(storage).unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).unwrap());

    let mut iter = AsIterator::<(), LedgerIndex>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
