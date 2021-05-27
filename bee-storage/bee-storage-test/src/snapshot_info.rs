// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::snapshot::SnapshotInfo;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::snapshot::rand_snapshot_info;

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(), SnapshotInfo>
    + Fetch<(), SnapshotInfo>
    + Insert<(), SnapshotInfo>
    + Delete<(), SnapshotInfo>
    + BatchBuilder
    + Batch<(), SnapshotInfo>
    + for<'a> AsStream<'a, (), SnapshotInfo>
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
        + for<'a> AsStream<'a, (), SnapshotInfo>
        + Truncate<(), SnapshotInfo>
{
}

pub async fn snapshot_info_access<B: StorageBackend>(storage: &B) {
    let snapshot_info = rand_snapshot_info();

    assert!(!Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(storage, &()).await.unwrap().is_none());

    Insert::<(), SnapshotInfo>::insert(storage, &(), &snapshot_info)
        .await
        .unwrap();

    assert!(Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), SnapshotInfo>::fetch(storage, &()).await.unwrap().unwrap(),
        snapshot_info
    );

    Delete::<(), SnapshotInfo>::delete(storage, &()).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(storage, &()).await.unwrap().is_none());

    let mut batch = B::batch_begin();

    Batch::<(), SnapshotInfo>::batch_insert(storage, &mut batch, &(), &snapshot_info).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), SnapshotInfo>::fetch(storage, &()).await.unwrap().unwrap(),
        snapshot_info
    );

    let mut batch = B::batch_begin();

    Batch::<(), SnapshotInfo>::batch_delete(storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(storage, &()).await.unwrap().is_none());

    Insert::<(), SnapshotInfo>::insert(storage, &(), &snapshot_info)
        .await
        .unwrap();

    let mut stream = AsStream::<(), SnapshotInfo>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (_, info) = result.unwrap();
        assert_eq!(snapshot_info, info);
        count += 1;
    }

    assert_eq!(count, 1);

    Truncate::<(), SnapshotInfo>::truncate(storage).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(storage, &()).await.unwrap());
}
