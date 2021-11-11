// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{address::AliasAddress, output::OutputId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{address::rand_alias_address, output::rand_output_id};

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(AliasAddress, OutputId), ()>
    + Fetch<AliasAddress, Vec<OutputId>>
    + Insert<(AliasAddress, OutputId), ()>
    + Delete<(AliasAddress, OutputId), ()>
    + BatchBuilder
    + Batch<(AliasAddress, OutputId), ()>
    + for<'a> AsIterator<'a, (AliasAddress, OutputId), ()>
    + Truncate<(AliasAddress, OutputId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(AliasAddress, OutputId), ()>
        + Fetch<AliasAddress, Vec<OutputId>>
        + Insert<(AliasAddress, OutputId), ()>
        + Delete<(AliasAddress, OutputId), ()>
        + BatchBuilder
        + Batch<(AliasAddress, OutputId), ()>
        + for<'a> AsIterator<'a, (AliasAddress, OutputId), ()>
        + Truncate<(AliasAddress, OutputId), ()>
{
}

pub fn alias_address_to_output_id_access<B: StorageBackend>(storage: &B) {
    let (address, output_id) = (rand_alias_address(), rand_output_id());

    assert!(!Exist::<(AliasAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<AliasAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(AliasAddress, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();

    assert!(Exist::<(AliasAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert_eq!(
        Fetch::<AliasAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap(),
        vec![output_id]
    );

    Delete::<(AliasAddress, OutputId), ()>::delete(storage, &(address, output_id)).unwrap();

    assert!(!Exist::<(AliasAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<AliasAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (address, output_id) = (rand_alias_address(), rand_output_id());
        Insert::<(AliasAddress, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();
        Batch::<(AliasAddress, OutputId), ()>::batch_delete(storage, &mut batch, &(address, output_id)).unwrap();
    }

    let mut output_ids = HashMap::<AliasAddress, Vec<OutputId>>::new();

    for _ in 0..5 {
        let address = rand_alias_address();
        for _ in 0..5 {
            let output_id = rand_output_id();
            Batch::<(AliasAddress, OutputId), ()>::batch_insert(storage, &mut batch, &(address, output_id), &())
                .unwrap();
            output_ids.entry(address).or_default().push(output_id);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(AliasAddress, OutputId), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((address, output_id), _) = result.unwrap();
        assert!(output_ids.get(&address).unwrap().contains(&output_id));
        count += 1;
    }

    assert_eq!(count, output_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(AliasAddress, OutputId), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(AliasAddress, OutputId), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
