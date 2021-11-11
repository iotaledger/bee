// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{address::NftAddress, output::OutputId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{address::rand_nft_address, output::rand_output_id};

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(NftAddress, OutputId), ()>
    + Fetch<NftAddress, Vec<OutputId>>
    + Insert<(NftAddress, OutputId), ()>
    + Delete<(NftAddress, OutputId), ()>
    + BatchBuilder
    + Batch<(NftAddress, OutputId), ()>
    + for<'a> AsIterator<'a, (NftAddress, OutputId), ()>
    + Truncate<(NftAddress, OutputId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(NftAddress, OutputId), ()>
        + Fetch<NftAddress, Vec<OutputId>>
        + Insert<(NftAddress, OutputId), ()>
        + Delete<(NftAddress, OutputId), ()>
        + BatchBuilder
        + Batch<(NftAddress, OutputId), ()>
        + for<'a> AsIterator<'a, (NftAddress, OutputId), ()>
        + Truncate<(NftAddress, OutputId), ()>
{
}

pub fn nft_address_to_output_id_access<B: StorageBackend>(storage: &B) {
    let (address, output_id) = (rand_nft_address(), rand_output_id());

    assert!(!Exist::<(NftAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<NftAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(NftAddress, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();

    assert!(Exist::<(NftAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert_eq!(
        Fetch::<NftAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap(),
        vec![output_id]
    );

    Delete::<(NftAddress, OutputId), ()>::delete(storage, &(address, output_id)).unwrap();

    assert!(!Exist::<(NftAddress, OutputId), ()>::exist(storage, &(address, output_id)).unwrap());
    assert!(
        Fetch::<NftAddress, Vec<OutputId>>::fetch(storage, &address)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (address, output_id) = (rand_nft_address(), rand_output_id());
        Insert::<(NftAddress, OutputId), ()>::insert(storage, &(address, output_id), &()).unwrap();
        Batch::<(NftAddress, OutputId), ()>::batch_delete(storage, &mut batch, &(address, output_id)).unwrap();
    }

    let mut output_ids = HashMap::<NftAddress, Vec<OutputId>>::new();

    for _ in 0..5 {
        let address = rand_nft_address();
        for _ in 0..5 {
            let output_id = rand_output_id();
            Batch::<(NftAddress, OutputId), ()>::batch_insert(storage, &mut batch, &(address, output_id), &()).unwrap();
            output_ids.entry(address).or_default().push(output_id);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(NftAddress, OutputId), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((address, output_id), _) = result.unwrap();
        assert!(output_ids.get(&address).unwrap().contains(&output_id));
        count += 1;
    }

    assert_eq!(count, output_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(NftAddress, OutputId), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(NftAddress, OutputId), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
