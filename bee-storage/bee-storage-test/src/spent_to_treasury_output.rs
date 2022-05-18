// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_ledger::types::TreasuryOutput;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::{bool::rand_bool, output::rand_ledger_treasury_output};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(bool, TreasuryOutput), ()>
    + Fetch<bool, Vec<TreasuryOutput>>
    + Insert<(bool, TreasuryOutput), ()>
    + Delete<(bool, TreasuryOutput), ()>
    + BatchBuilder
    + Batch<(bool, TreasuryOutput), ()>
    + for<'a> AsIterator<'a, (bool, TreasuryOutput), ()>
    + Truncate<(bool, TreasuryOutput), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(bool, TreasuryOutput), ()>
        + Fetch<bool, Vec<TreasuryOutput>>
        + Insert<(bool, TreasuryOutput), ()>
        + Delete<(bool, TreasuryOutput), ()>
        + BatchBuilder
        + Batch<(bool, TreasuryOutput), ()>
        + for<'a> AsIterator<'a, (bool, TreasuryOutput), ()>
        + Truncate<(bool, TreasuryOutput), ()>
{
}

pub fn spent_to_treasury_output_access<B: StorageBackend>(storage: &B) {
    let (spent, treasury_output) = (rand_bool(), rand_ledger_treasury_output());

    assert!(
        !storage
            .exist::<(bool, TreasuryOutput), ()>(&(spent, treasury_output.clone()))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<bool, Vec<TreasuryOutput>>(&spent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    storage.insert(&(spent, treasury_output.clone()), &()).unwrap();

    assert!(
        storage
            .exist::<(bool, TreasuryOutput), ()>(&(spent, treasury_output.clone()))
            .unwrap()
    );
    assert_eq!(
        storage.fetch::<bool, Vec<TreasuryOutput>>(&spent).unwrap().unwrap(),
        vec![treasury_output.clone()]
    );

    storage
        .delete::<(bool, TreasuryOutput), ()>(&(spent, treasury_output.clone()))
        .unwrap();

    assert!(
        !storage
            .exist::<(bool, TreasuryOutput), ()>(&(spent, treasury_output))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<bool, Vec<TreasuryOutput>>(&spent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (spent, treasury_output) = (rand_bool(), rand_ledger_treasury_output());
        storage.insert(&(spent, treasury_output.clone()), &()).unwrap();
        storage
            .batch_delete::<(bool, TreasuryOutput), ()>(&mut batch, &(spent, treasury_output))
            .unwrap();
    }

    let mut treasury_outputs = HashMap::<bool, Vec<TreasuryOutput>>::new();

    for _ in 0..10 {
        let spent = false;
        let treasury_output = rand_ledger_treasury_output();
        storage
            .batch_insert(&mut batch, &(spent, treasury_output.clone()), &())
            .unwrap();
        treasury_outputs.entry(spent).or_default().push(treasury_output);
    }

    for _ in 0..10 {
        let spent = true;
        let treasury_output = rand_ledger_treasury_output();
        storage
            .batch_insert(&mut batch, &(spent, treasury_output.clone()), &())
            .unwrap();
        treasury_outputs.entry(spent).or_default().push(treasury_output);
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = storage.iter::<(bool, TreasuryOutput), ()>().unwrap();
    let mut count = 0;

    for result in iter {
        let ((spent, treasury_output), _) = result.unwrap();
        assert!(treasury_outputs.get(&spent).unwrap().contains(&treasury_output));
        count += 1;
    }

    assert_eq!(count, treasury_outputs.iter().fold(0, |acc, v| acc + v.1.len()));

    storage.truncate::<(bool, TreasuryOutput), ()>().unwrap();

    let mut iter = storage.iter::<(bool, TreasuryOutput), ()>().unwrap();

    assert!(iter.next().is_none());
}
