// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Unspent;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Insert, Truncate},
    backend,
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

    assert!(!Exist::<Unspent, ()>::exist(storage, &unspent).unwrap());

    Insert::<Unspent, ()>::insert(storage, &unspent, &()).unwrap();

    assert!(Exist::<Unspent, ()>::exist(storage, &unspent).unwrap());

    Delete::<Unspent, ()>::delete(storage, &unspent).unwrap();

    assert!(!Exist::<Unspent, ()>::exist(storage, &unspent).unwrap());

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let unspent = rand_unspent_output_id();
        Insert::<Unspent, ()>::insert(storage, &unspent, &()).unwrap();
        Batch::<Unspent, ()>::batch_delete(storage, &mut batch, &unspent).unwrap();
    }

    let mut unspents = Vec::new();

    for _ in 0..10 {
        let unspent = rand_unspent_output_id();
        Batch::<Unspent, ()>::batch_insert(storage, &mut batch, &unspent, &()).unwrap();
        unspents.push(unspent);
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<Unspent, ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (unspent, ()) = result.unwrap();
        assert!(unspents.contains(&unspent));
        count += 1;
    }

    assert_eq!(count, unspents.len());

    Truncate::<Unspent, ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<Unspent, ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
