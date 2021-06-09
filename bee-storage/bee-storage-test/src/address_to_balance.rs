// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::Balance;
use bee_message::address::Address;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::{address::rand_address, balance::rand_balance};

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<Address, Balance>
    + Fetch<Address, Balance>
    + MultiFetch<Address, Balance>
    + Insert<Address, Balance>
    + Delete<Address, Balance>
    + BatchBuilder
    + Batch<Address, Balance>
    + for<'a> AsStream<'a, Address, Balance>
    + Truncate<Address, Balance>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<Address, Balance>
        + Fetch<Address, Balance>
        + MultiFetch<Address, Balance>
        + Insert<Address, Balance>
        + Delete<Address, Balance>
        + BatchBuilder
        + Batch<Address, Balance>
        + for<'a> AsStream<'a, Address, Balance>
        + Truncate<Address, Balance>
{
}

pub async fn address_to_balance_access<B: StorageBackend>(storage: &B) {
    let (address, balance) = (rand_address(), rand_balance());

    assert!(!Exist::<Address, Balance>::exist(storage, &address).await.unwrap());
    assert!(
        Fetch::<Address, Balance>::fetch(storage, &address)
            .await
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<Address, Balance>::multi_fetch(storage, &[address])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<Address, Balance>::insert(storage, &address, &balance)
        .await
        .unwrap();

    assert!(Exist::<Address, Balance>::exist(storage, &address).await.unwrap());
    assert_eq!(
        Fetch::<Address, Balance>::fetch(storage, &address)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        balance.pack_new()
    );
    let results = MultiFetch::<Address, Balance>::multi_fetch(storage, &[address])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &balance));

    Delete::<Address, Balance>::delete(storage, &address).await.unwrap();

    assert!(!Exist::<Address, Balance>::exist(storage, &address).await.unwrap());
    assert!(
        Fetch::<Address, Balance>::fetch(storage, &address)
            .await
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<Address, Balance>::multi_fetch(storage, &[address])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut addresses = Vec::new();
    let mut balances = Vec::new();

    for _ in 0..10 {
        let (address, balance) = (rand_address(), rand_balance());
        Insert::<Address, Balance>::insert(storage, &address, &balance)
            .await
            .unwrap();
        Batch::<Address, Balance>::batch_delete(storage, &mut batch, &address).unwrap();
        addresses.push(address);
        balances.push((address, None));
    }

    for _ in 0..10 {
        let (address, balance) = (rand_address(), rand_balance());
        Batch::<Address, Balance>::batch_insert(storage, &mut batch, &address, &balance).unwrap();
        addresses.push(address);
        balances.push((address, Some(balance)));
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<Address, Balance>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (address, balance) = result.unwrap();
        assert!(balances.contains(&(address, Some(balance))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<Address, Balance>::multi_fetch(storage, &addresses)
        .await
        .unwrap();

    assert_eq!(results.len(), addresses.len());

    for ((_, balance), result) in balances.into_iter().zip(results.into_iter()) {
        assert_eq!(balance, result.unwrap());
    }

    Truncate::<Address, Balance>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<Address, Balance>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
