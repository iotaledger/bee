// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::TreasuryOutput;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{bool::rand_bool, treasury_output::rand_treasury_output};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(bool, TreasuryOutput), ()>
    + Fetch<bool, Vec<TreasuryOutput>>
    + Insert<(bool, TreasuryOutput), ()>
    + Delete<(bool, TreasuryOutput), ()>
    + BatchBuilder
    + Batch<(bool, TreasuryOutput), ()>
    + for<'a> AsStream<'a, (bool, TreasuryOutput), ()>
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
        + for<'a> AsStream<'a, (bool, TreasuryOutput), ()>
        + Truncate<(bool, TreasuryOutput), ()>
{
}

pub async fn spent_to_treasury_output_access<B: StorageBackend>(storage: &B) {
    let (spent, treasury_output) = (rand_bool(), rand_treasury_output());

    assert!(
        !Exist::<(bool, TreasuryOutput), ()>::exist(storage, &(spent, treasury_output.clone()))
            .await
            .unwrap()
    );
    assert!(Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &spent)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(bool, TreasuryOutput), ()>::insert(storage, &(spent, treasury_output.clone()), &())
        .await
        .unwrap();

    assert!(
        Exist::<(bool, TreasuryOutput), ()>::exist(storage, &(spent, treasury_output.clone()))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &spent)
            .await
            .unwrap()
            .unwrap(),
        vec![treasury_output.clone()]
    );

    Delete::<(bool, TreasuryOutput), ()>::delete(storage, &(spent, treasury_output.clone()))
        .await
        .unwrap();

    assert!(
        !Exist::<(bool, TreasuryOutput), ()>::exist(storage, &(spent, treasury_output.clone()))
            .await
            .unwrap()
    );
    assert!(Fetch::<bool, Vec<TreasuryOutput>>::fetch(storage, &spent)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (spent, treasury_output) = (rand_bool(), rand_treasury_output());
        Insert::<(bool, TreasuryOutput), ()>::insert(storage, &(spent, treasury_output.clone()), &())
            .await
            .unwrap();
        Batch::<(bool, TreasuryOutput), ()>::batch_delete(storage, &mut batch, &(spent, treasury_output)).unwrap();
    }

    let mut treasury_outputs = HashMap::<bool, Vec<TreasuryOutput>>::new();

    for _ in 0usize..10usize {
        let spent = false;
        let treasury_output = rand_treasury_output();
        Batch::<(bool, TreasuryOutput), ()>::batch_insert(storage, &mut batch, &(spent, treasury_output.clone()), &())
            .unwrap();
        treasury_outputs.entry(spent).or_default().push(treasury_output);
    }

    for _ in 0usize..10usize {
        let spent = true;
        let treasury_output = rand_treasury_output();
        Batch::<(bool, TreasuryOutput), ()>::batch_insert(storage, &mut batch, &(spent, treasury_output.clone()), &())
            .unwrap();
        treasury_outputs.entry(spent).or_default().push(treasury_output);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(bool, TreasuryOutput), ()>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(((spent, treasury_output), _)) = stream.next().await {
        assert!(treasury_outputs.get(&spent).unwrap().contains(&treasury_output));
        count += 1;
    }

    assert_eq!(count, treasury_outputs.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(bool, TreasuryOutput), ()>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<(bool, TreasuryOutput), ()>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
