// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::LedgerIndex;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(), LedgerIndex>
    + Fetch<(), LedgerIndex>
    + Insert<(), LedgerIndex>
    + Delete<(), LedgerIndex>
    + BatchBuilder
    + Batch<(), LedgerIndex>
    + for<'a> AsStream<'a, (), LedgerIndex>
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
        + for<'a> AsStream<'a, (), LedgerIndex>
        + Truncate<(), LedgerIndex>
{
}

pub async fn ledger_index_access<B: StorageBackend>(storage: &B) {
    let index = LedgerIndex::from(MilestoneIndex::from(42));

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).await.unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(storage, &(), &index).await.unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), LedgerIndex>::fetch(storage, &()).await.unwrap().unwrap(),
        index
    );

    Delete::<(), LedgerIndex>::delete(storage, &()).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).await.unwrap().is_none());

    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_insert(storage, &mut batch, &(), &index).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
    assert_eq!(
        Fetch::<(), LedgerIndex>::fetch(storage, &()).await.unwrap().unwrap(),
        index
    );

    let mut batch = B::batch_begin();

    Batch::<(), LedgerIndex>::batch_delete(storage, &mut batch, &()).unwrap();

    storage.batch_commit(batch, true).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
    assert!(Fetch::<(), LedgerIndex>::fetch(storage, &()).await.unwrap().is_none());

    Insert::<(), LedgerIndex>::insert(storage, &(), &index).await.unwrap();

    let mut stream = AsStream::<(), LedgerIndex>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (_, ledger_index) = result.unwrap();
        assert_eq!(ledger_index, index);
        count += 1;
    }

    assert_eq!(count, 1);

    Truncate::<(), LedgerIndex>::truncate(storage).await.unwrap();

    assert!(!Exist::<(), LedgerIndex>::exist(storage, &()).await.unwrap());
}
