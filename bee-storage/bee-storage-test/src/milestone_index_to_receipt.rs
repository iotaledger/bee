// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_block::payload::milestone::MilestoneIndex;
use bee_ledger::types::Receipt;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::{milestone::rand_milestone_index, receipt::rand_ledger_receipt};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MilestoneIndex, Receipt), ()>
    + Fetch<MilestoneIndex, Vec<Receipt>>
    + Insert<(MilestoneIndex, Receipt), ()>
    + Delete<(MilestoneIndex, Receipt), ()>
    + BatchBuilder
    + Batch<(MilestoneIndex, Receipt), ()>
    + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
    + Truncate<(MilestoneIndex, Receipt), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MilestoneIndex, Receipt), ()>
        + Fetch<MilestoneIndex, Vec<Receipt>>
        + Insert<(MilestoneIndex, Receipt), ()>
        + Delete<(MilestoneIndex, Receipt), ()>
        + BatchBuilder
        + Batch<(MilestoneIndex, Receipt), ()>
        + for<'a> AsIterator<'a, (MilestoneIndex, Receipt), ()>
        + Truncate<(MilestoneIndex, Receipt), ()>
{
}

pub fn milestone_index_to_receipt_access<B: StorageBackend>(storage: &B) {
    let (index, receipt) = (rand_milestone_index(), rand_ledger_receipt());

    assert!(!Exist::<(MilestoneIndex, Receipt), ()>::exist(storage, &(index, receipt.clone())).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(MilestoneIndex, Receipt), ()>::insert(storage, &(index, receipt.clone()), &()).unwrap();

    assert!(Exist::<(MilestoneIndex, Receipt), ()>::exist(storage, &(index, receipt.clone())).unwrap());
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
            .unwrap()
            .unwrap(),
        vec![receipt.clone()]
    );

    Delete::<(MilestoneIndex, Receipt), ()>::delete(storage, &(index, receipt.clone())).unwrap();

    assert!(!Exist::<(MilestoneIndex, Receipt), ()>::exist(storage, &(index, receipt)).unwrap());
    assert!(
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, receipt) = (rand_milestone_index(), rand_ledger_receipt());
        Insert::<(MilestoneIndex, Receipt), ()>::insert(storage, &(index, receipt.clone()), &()).unwrap();
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, &mut batch, &(index, receipt)).unwrap();
    }

    let mut receipts = HashMap::<MilestoneIndex, Vec<Receipt>>::new();

    for _ in 0..5 {
        let index = rand_milestone_index();
        for _ in 0..5 {
            let receipt = rand_ledger_receipt();
            Batch::<(MilestoneIndex, Receipt), ()>::batch_insert(storage, &mut batch, &(index, receipt.clone()), &())
                .unwrap();
            receipts.entry(index).or_default().push(receipt);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, block_id), _) = result.unwrap();
        assert!(receipts.get(&index).unwrap().contains(&block_id));
        count += 1;
    }

    assert_eq!(count, receipts.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, Receipt), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(MilestoneIndex, Receipt), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
