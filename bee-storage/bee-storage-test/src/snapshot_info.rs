// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::snapshot::SnapshotInfo;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::snapshot::rand_snapshot_info;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(), SnapshotInfo>
    + Fetch<(), SnapshotInfo>
    + Insert<(), SnapshotInfo>
    + Delete<(), SnapshotInfo>
    + BatchBuilder
    + Batch<(), SnapshotInfo>
    + for<'a> AsIterator<'a, (), SnapshotInfo>
    + Truncate<(), SnapshotInfo>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(), SnapshotInfo>
        + Fetch<(), SnapshotInfo>
        + Insert<(), SnapshotInfo>
        + Delete<(), SnapshotInfo>
        + BatchBuilder
        + Batch<(), SnapshotInfo>
        + for<'a> AsIterator<'a, (), SnapshotInfo>
        + Truncate<(), SnapshotInfo>
{
}

pub fn snapshot_info_access<B: StorageBackend>(storage: &B) {
    let snapshot_info = rand_snapshot_info();

    assert!(!storage.exist::<(), SnapshotInfo>(&()).unwrap());
    assert!(storage.fetch::<(), SnapshotInfo>(&()).unwrap().is_none());

    storage.insert(&(), &snapshot_info).unwrap();

    assert!(storage.exist::<(), SnapshotInfo>(&()).unwrap());
    assert_eq!(storage.fetch::<(), SnapshotInfo>(&()).unwrap().unwrap(), snapshot_info);

    storage.delete::<(), SnapshotInfo>(&()).unwrap();

    assert!(!storage.exist::<(), SnapshotInfo>(&()).unwrap());
    assert!(storage.fetch::<(), SnapshotInfo>(&()).unwrap().is_none());

    let mut batch = B::batch_begin();

    storage.batch_insert(&mut batch, &(), &snapshot_info).unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(storage.exist::<(), SnapshotInfo>(&()).unwrap());
    assert_eq!(storage.fetch::<(), SnapshotInfo>(&()).unwrap().unwrap(), snapshot_info);

    let mut batch = B::batch_begin();

    storage.batch_delete::<(), SnapshotInfo>(&mut batch, &()).unwrap();

    storage.batch_commit(batch, true).unwrap();

    assert!(!storage.exist::<(), SnapshotInfo>(&()).unwrap());
    assert!(storage.fetch::<(), SnapshotInfo>(&()).unwrap().is_none());

    storage.insert(&(), &snapshot_info).unwrap();

    let iter = storage.iter::<(), SnapshotInfo>().unwrap();
    let mut count = 0;

    for result in iter {
        let (_, info) = result.unwrap();
        assert_eq!(snapshot_info, info);
        count += 1;
    }

    assert_eq!(count, 1);

    storage.truncate::<(), SnapshotInfo>().unwrap();

    assert!(!storage.exist::<(), SnapshotInfo>(&()).unwrap());

    let mut iter = storage.iter::<(), SnapshotInfo>().unwrap();

    assert!(iter.next().is_none());
}
