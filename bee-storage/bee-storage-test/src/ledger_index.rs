// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::LedgerIndex;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
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

    assert!(!storage.exist::<(), LedgerIndex>(&()).unwrap());
    assert!(storage.fetch::<(), LedgerIndex>(&()).unwrap().is_none());

    storage.insert::<(), LedgerIndex>(&(), &index).unwrap();

    assert!(storage.exist::<(), LedgerIndex>(&()).unwrap());
    assert_eq!(storage.fetch::<(), LedgerIndex>(&()).unwrap().unwrap(), index);

    storage.delete::<(), LedgerIndex>(&()).unwrap();

    assert!(!storage.exist::<(), LedgerIndex>(&()).unwrap());
    assert!(storage.fetch::<(), LedgerIndex>(&()).unwrap().is_none());

    let mut batch = B::batch_begin();

    storage
        .batch_insert::<(), LedgerIndex>(&mut batch, &(), &index)
        .unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(storage.exist::<(), LedgerIndex>(&()).unwrap());
    assert_eq!(storage.fetch::<(), LedgerIndex>(&()).unwrap().unwrap(), index);

    let mut batch = B::batch_begin();

    storage.batch_delete::<(), LedgerIndex>(&mut batch, &()).unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(!storage.exist::<(), LedgerIndex>(&()).unwrap());
    assert!(storage.fetch::<(), LedgerIndex>(&()).unwrap().is_none());

    storage.insert::<(), LedgerIndex>(&(), &index).unwrap();

    let iter = storage.iter::<(), LedgerIndex>().unwrap();
    let mut count = 0;

    for result in iter {
        let (_, ledger_index) = result.unwrap();
        assert_eq!(ledger_index, index);
        count += 1;
    }

    assert_eq!(count, 1);

    storage.truncate::<(), LedgerIndex>().unwrap();

    assert!(!storage.exist::<(), LedgerIndex>(&()).unwrap());

    let mut iter = storage.iter::<(), LedgerIndex>().unwrap();

    assert!(iter.next().is_none());
}
