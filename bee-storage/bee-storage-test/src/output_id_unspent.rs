// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Unspent;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Insert, Truncate},
    backend::{self, StorageBackendExt},
};
use bee_test::rand::output::rand_unspent_output_id;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<Unspent, ()>
    + Insert<Unspent, ()>
    + Delete<Unspent, ()>
    + BatchBuilder
    + Batch<Unspent, ()>
    + for<'a> AsIterator<'a, Unspent, ()>
    + Truncate<Unspent, ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<Unspent, ()>
        + Insert<Unspent, ()>
        + Delete<Unspent, ()>
        + BatchBuilder
        + Batch<Unspent, ()>
        + for<'a> AsIterator<'a, Unspent, ()>
        + Truncate<Unspent, ()>
{
}

pub fn output_id_unspent_access<B: StorageBackend>(storage: &B) {
    let unspent = rand_unspent_output_id();

    assert!(!storage.exist::<Unspent, ()>(&unspent).unwrap());

    storage.insert(&unspent, &()).unwrap();

    assert!(storage.exist::<Unspent, ()>(&unspent).unwrap());

    storage.delete::<Unspent, ()>(&unspent).unwrap();

    assert!(!storage.exist::<Unspent, ()>(&unspent).unwrap());

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let unspent = rand_unspent_output_id();
        storage.insert(&unspent, &()).unwrap();
        storage.batch_delete::<Unspent, ()>(&mut batch, &unspent).unwrap();
    }

    let mut unspents = Vec::new();

    for _ in 0..10 {
        let unspent = rand_unspent_output_id();
        storage.batch_insert(&mut batch, &unspent, &()).unwrap();
        unspents.push(unspent);
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<Unspent, ()>().unwrap();
    let mut count = 0;

    for result in iter {
        let (unspent, ()) = result.unwrap();
        assert!(unspents.contains(&unspent));
        count += 1;
    }

    assert_eq!(count, unspents.len());

    storage.truncate::<Unspent, ()>().unwrap();

    let mut iter = storage.iter::<Unspent, ()>().unwrap();

    assert!(iter.next().is_none());
}
