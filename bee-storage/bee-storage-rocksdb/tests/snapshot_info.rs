// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_snapshot::info::SnapshotInfo;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDbConfigBuilder, storage::Storage};
use bee_test::rand::snapshot::rand_snapshot_info;

use futures::stream::StreamExt;

const DB_DIRECTORY: &str = "./tests/database/snapshot_info";

#[tokio::test]
async fn snapshot_info_access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDbConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let snapshot_info = rand_snapshot_info();

    assert!(!Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(&storage, &()).await.unwrap().is_none());

    Insert::<(), SnapshotInfo>::insert(&storage, &(), &snapshot_info)
        .await
        .unwrap();

    assert!(Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), SnapshotInfo>::fetch(&storage, &()).await.unwrap().unwrap(),
        snapshot_info
    );

    Delete::<(), SnapshotInfo>::delete(&storage, &()).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(&storage, &()).await.unwrap().is_none());

    let mut batch = Storage::batch_begin();

    Batch::<(), SnapshotInfo>::batch_insert(&storage, &mut batch, &(), &snapshot_info).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), SnapshotInfo>::fetch(&storage, &()).await.unwrap().unwrap(),
        snapshot_info
    );

    let mut batch = Storage::batch_begin();

    Batch::<(), SnapshotInfo>::batch_delete(&storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());
    assert!(Fetch::<(), SnapshotInfo>::fetch(&storage, &()).await.unwrap().is_none());

    Insert::<(), SnapshotInfo>::insert(&storage, &(), &snapshot_info)
        .await
        .unwrap();

    let mut stream = AsStream::<(), SnapshotInfo>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((_, info)) = stream.next().await {
        assert_eq!(snapshot_info, info);
        count += 1;
    }

    assert_eq!(count, 1);

    Truncate::<(), SnapshotInfo>::truncate(&storage).await.unwrap();

    assert!(!Exist::<(), SnapshotInfo>::exist(&storage, &()).await.unwrap());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
