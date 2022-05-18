// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::Balance;
use bee_message::address::Address;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::{address::rand_address, balance::rand_balance};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<Address, Balance>
    + Fetch<Address, Balance>
    + for<'a> MultiFetch<'a, Address, Balance>
    + Insert<Address, Balance>
    + Delete<Address, Balance>
    + BatchBuilder
    + Batch<Address, Balance>
    + for<'a> AsIterator<'a, Address, Balance>
    + Truncate<Address, Balance>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<Address, Balance>
        + Fetch<Address, Balance>
        + for<'a> MultiFetch<'a, Address, Balance>
        + Insert<Address, Balance>
        + Delete<Address, Balance>
        + BatchBuilder
        + Batch<Address, Balance>
        + for<'a> AsIterator<'a, Address, Balance>
        + Truncate<Address, Balance>
{
}

pub fn address_to_balance_access<B: StorageBackend>(storage: &B) {
    let (address, balance) = (rand_address(), rand_balance());

    assert!(!storage.exist::<Address, Balance>(&address).unwrap());
    assert!(storage.fetch::<Address, Balance>(&address).unwrap().is_none());
    let results = storage
        .multi_fetch::<Address, Balance>(&[address])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    storage.insert::<Address, Balance>(&address, &balance).unwrap();

    assert!(storage.exist::<Address, Balance>(&address).unwrap());
    assert_eq!(
        storage.fetch::<Address, Balance>(&address).unwrap().unwrap().pack_new(),
        balance.pack_new()
    );
    let results = storage
        .multi_fetch::<Address, Balance>(&[address])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &balance));

    storage.delete::<Address, Balance>(&address).unwrap();

    assert!(!storage.exist::<Address, Balance>(&address).unwrap());
    assert!(storage.fetch::<Address, Balance>(&address).unwrap().is_none());
    let results = storage
        .multi_fetch::<Address, Balance>(&[address])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut addresses = Vec::new();
    let mut balances = Vec::new();

    for _ in 0..10 {
        let (address, balance) = (rand_address(), rand_balance());
        storage.insert::<Address, Balance>(&address, &balance).unwrap();
        storage.batch_delete::<Address, Balance>(&mut batch, &address).unwrap();
        addresses.push(address);
        balances.push((address, None));
    }

    for _ in 0..10 {
        let (address, balance) = (rand_address(), rand_balance());
        storage
            .batch_insert::<Address, Balance>(&mut batch, &address, &balance)
            .unwrap();
        addresses.push(address);
        balances.push((address, Some(balance)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<Address, Balance>().unwrap();
    let mut count = 0;

    for result in iter {
        let (address, balance) = result.unwrap();
        assert!(balances.contains(&(address, Some(balance))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = storage
        .multi_fetch::<Address, Balance>(&addresses)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), addresses.len());

    for ((_, balance), result) in balances.into_iter().zip(results.into_iter()) {
        assert_eq!(balance, result.unwrap());
    }

    Truncate::<Address, Balance>::truncate_op(storage).unwrap();

    let mut iter = storage.iter::<Address, Balance>().unwrap();

    assert!(iter.next().is_none());
}
