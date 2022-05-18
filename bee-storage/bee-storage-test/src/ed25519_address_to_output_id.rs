// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::{address::Ed25519Address, output::OutputId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::{address::rand_ed25519_address, output::rand_output_id};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(Ed25519Address, OutputId), ()>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Insert<(Ed25519Address, OutputId), ()>
    + Delete<(Ed25519Address, OutputId), ()>
    + BatchBuilder
    + Batch<(Ed25519Address, OutputId), ()>
    + for<'a> AsIterator<'a, (Ed25519Address, OutputId), ()>
    + Truncate<(Ed25519Address, OutputId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(Ed25519Address, OutputId), ()>
        + Fetch<Ed25519Address, Vec<OutputId>>
        + Insert<(Ed25519Address, OutputId), ()>
        + Delete<(Ed25519Address, OutputId), ()>
        + BatchBuilder
        + Batch<(Ed25519Address, OutputId), ()>
        + for<'a> AsIterator<'a, (Ed25519Address, OutputId), ()>
        + Truncate<(Ed25519Address, OutputId), ()>
{
}

pub fn ed25519_address_to_output_id_access<B: StorageBackend>(storage: &B) {
    let (address, output_id) = (rand_ed25519_address(), rand_output_id());

    assert!(
        !storage
            .exist::<(Ed25519Address, OutputId), ()>(&(address, output_id))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<Ed25519Address, Vec<OutputId>>(&address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    storage
        .insert::<(Ed25519Address, OutputId), ()>(&(address, output_id), &())
        .unwrap();

    assert!(
        storage
            .exist::<(Ed25519Address, OutputId), ()>(&(address, output_id))
            .unwrap()
    );
    assert_eq!(
        storage
            .fetch::<Ed25519Address, Vec<OutputId>>(&address)
            .unwrap()
            .unwrap(),
        vec![output_id]
    );

    storage
        .delete::<(Ed25519Address, OutputId), ()>(&(address, output_id))
        .unwrap();

    assert!(
        !storage
            .exist::<(Ed25519Address, OutputId), ()>(&(address, output_id))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<Ed25519Address, Vec<OutputId>>(&address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (address, output_id) = (rand_ed25519_address(), rand_output_id());
        storage
            .insert::<(Ed25519Address, OutputId), ()>(&(address, output_id), &())
            .unwrap();
        storage
            .batch_delete::<(Ed25519Address, OutputId), ()>(&mut batch, &(address, output_id))
            .unwrap();
    }

    let mut output_ids = HashMap::<Ed25519Address, Vec<OutputId>>::new();

    for _ in 0..5 {
        let address = rand_ed25519_address();
        for _ in 0..5 {
            let output_id = rand_output_id();
            storage
                .batch_insert::<(Ed25519Address, OutputId), ()>(&mut batch, &(address, output_id), &())
                .unwrap();
            output_ids.entry(address).or_default().push(output_id);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<(Ed25519Address, OutputId), ()>().unwrap();
    let mut count = 0;

    for result in iter {
        let ((address, output_id), _) = result.unwrap();
        assert!(output_ids.get(&address).unwrap().contains(&output_id));
        count += 1;
    }

    assert_eq!(count, output_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(Ed25519Address, OutputId), ()>::truncate_op(storage).unwrap();

    let mut iter = storage.iter::<(Ed25519Address, OutputId), ()>().unwrap();

    assert!(iter.next().is_none());
}
