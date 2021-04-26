// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::Balance;
use bee_message::address::Address;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{address::rand_address, balance::rand_balance};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<Address, Balance>
    + Fetch<Address, Balance>
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

    Delete::<Address, Balance>::delete(storage, &address).await.unwrap();

    assert!(!Exist::<Address, Balance>::exist(storage, &address).await.unwrap());
    assert!(
        Fetch::<Address, Balance>::fetch(storage, &address)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (address, balance) = (rand_address(), rand_balance());
        Insert::<Address, Balance>::insert(storage, &address, &balance)
            .await
            .unwrap();
        Batch::<Address, Balance>::batch_delete(storage, &mut batch, &address).unwrap();
    }

    let mut balances = HashMap::new();

    for _ in 0usize..10usize {
        let (address, balance) = (rand_address(), rand_balance());
        Batch::<Address, Balance>::batch_insert(storage, &mut batch, &address, &balance).unwrap();
        balances.insert(address, balance);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<Address, Balance>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((address, balance)) = stream.next().await {
        assert_eq!(balances.get(&address).unwrap().pack_new(), balance.pack_new());
        count += 1;
    }

    assert_eq!(count, balances.len());

    Truncate::<Address, Balance>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<Address, Balance>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
