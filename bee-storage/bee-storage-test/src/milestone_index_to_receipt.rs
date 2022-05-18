// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
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

    assert!(
        !storage
            .exist::<(MilestoneIndex, Receipt), ()>(&(index, receipt.clone()))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<MilestoneIndex, Vec<Receipt>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    storage
        .insert::<(MilestoneIndex, Receipt), ()>(&(index, receipt.clone()), &())
        .unwrap();

    assert!(
        storage
            .exist::<(MilestoneIndex, Receipt), ()>(&(index, receipt.clone()))
            .unwrap()
    );
    assert_eq!(
        storage.fetch::<MilestoneIndex, Vec<Receipt>>(&index).unwrap().unwrap(),
        vec![receipt.clone()]
    );

    storage
        .delete::<(MilestoneIndex, Receipt), ()>(&(index, receipt.clone()))
        .unwrap();

    assert!(
        !storage
            .exist::<(MilestoneIndex, Receipt), ()>(&(index, receipt))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<MilestoneIndex, Vec<Receipt>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, receipt) = (rand_milestone_index(), rand_ledger_receipt());
        storage
            .insert::<(MilestoneIndex, Receipt), ()>(&(index, receipt.clone()), &())
            .unwrap();
        storage
            .batch_delete::<(MilestoneIndex, Receipt), ()>(&mut batch, &(index, receipt))
            .unwrap();
    }

    let mut receipts = HashMap::<MilestoneIndex, Vec<Receipt>>::new();

    for _ in 0..5 {
        let index = rand_milestone_index();
        for _ in 0..5 {
            let receipt = rand_ledger_receipt();
            storage
                .batch_insert::<(MilestoneIndex, Receipt), ()>(&mut batch, &(index, receipt.clone()), &())
                .unwrap();
            receipts.entry(index).or_default().push(receipt);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(MilestoneIndex, Receipt), ()>::iter_op(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, message_id), _) = result.unwrap();
        assert!(receipts.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, receipts.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, Receipt), ()>::truncate_op(storage).unwrap();

    let mut iter = AsIterator::<(MilestoneIndex, Receipt), ()>::iter_op(storage).unwrap();

    assert!(iter.next().is_none());
}
