// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{address::Ed25519Address, output::OutputId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{address::rand_ed25519_address, output::rand_output_id};

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(Ed25519Address, OutputId), ()>
    + Fetch<Ed25519Address, Vec<OutputId>>
    + Insert<(Ed25519Address, OutputId), ()>
    + Delete<(Ed25519Address, OutputId), ()>
    + BatchBuilder
    + Batch<(Ed25519Address, OutputId), ()>
    + for<'a> AsStream<'a, (Ed25519Address, OutputId), ()>
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
        + for<'a> AsStream<'a, (Ed25519Address, OutputId), ()>
        + Truncate<(Ed25519Address, OutputId), ()>
{
}

pub fn ed25519_address_to_output_id_access<B: StorageBackend>(storage: &B) {
    let (address, output_id) = (rand_ed25519_address(), rand_output_id());

    assert!(!Exist::<(Ed25519Address, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(Ed25519Address, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();

    assert!(Exist::<(Ed25519Address, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert_eq!(
        Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap(),
        vec![output_id]
    );

    Delete::<(Ed25519Address, OutputId), ()>::delete(storage, &(address, output_id)).unwrap();

    assert!(!Exist::<(Ed25519Address, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<Ed25519Address, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (address, output_id) = (rand_ed25519_address(), rand_output_id());
        Insert::<(Ed25519Address, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();
        Batch::<(Ed25519Address, OutputId), ()>::batch_delete(storage, &mut batch, &(address, output_id)).unwrap();
    }

    let mut output_ids = HashMap::<Ed25519Address, Vec<OutputId>>::new();

    for _ in 0..5 {
        let address = rand_ed25519_address();
        for _ in 0..5 {
            let output_id = rand_output_id();
            Batch::<(Ed25519Address, OutputId), ()>::batch_insert(storage, &mut batch, &(address, output_id), &())
                .unwrap();
            output_ids.entry(address).or_default().push(output_id);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    // let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(storage).unwrap();
    // let mut count = 0;
    //
    // while let Some(result) = stream.next() {
    //     let ((address, output_id), _) = result.unwrap();
    //     assert!(output_ids.get(&address).unwrap().contains(&output_id));
    //     count += 1;
    // }
    //
    // assert_eq!(count, output_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(Ed25519Address, OutputId), ()>::truncate(storage).unwrap();

    // let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(storage).unwrap();
    //
    // assert!(stream.next().is_none());
}
