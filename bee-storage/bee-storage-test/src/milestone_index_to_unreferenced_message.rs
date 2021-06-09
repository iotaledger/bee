// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::unreferenced_message::UnreferencedMessage;
use bee_test::rand::{milestone::rand_milestone_index, unreferenced_message::rand_unreferenced_message};

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MilestoneIndex, UnreferencedMessage), ()>
    + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
    + Insert<(MilestoneIndex, UnreferencedMessage), ()>
    + Delete<(MilestoneIndex, UnreferencedMessage), ()>
    + BatchBuilder
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + for<'a> AsIterator<'a, (MilestoneIndex, UnreferencedMessage), ()>
    + Truncate<(MilestoneIndex, UnreferencedMessage), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MilestoneIndex, UnreferencedMessage), ()>
        + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
        + Insert<(MilestoneIndex, UnreferencedMessage), ()>
        + Delete<(MilestoneIndex, UnreferencedMessage), ()>
        + BatchBuilder
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + for<'a> AsIterator<'a, (MilestoneIndex, UnreferencedMessage), ()>
        + Truncate<(MilestoneIndex, UnreferencedMessage), ()>
{
}

pub fn milestone_index_to_unreferenced_message_access<B: StorageBackend>(storage: &B) {
    let (index, unreferenced_message) = (rand_milestone_index(), rand_unreferenced_message());

    assert!(
        !Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message)).unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(MilestoneIndex, UnreferencedMessage), ()>::insert(storage, &(index, unreferenced_message), &()).unwrap();

    assert!(
        Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message)).unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
            .unwrap()
            .unwrap(),
        vec![unreferenced_message]
    );

    Delete::<(MilestoneIndex, UnreferencedMessage), ()>::delete(storage, &(index, unreferenced_message)).unwrap();

    assert!(
        !Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message)).unwrap()
    );
    assert!(
        Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, unreferenced_message) = (rand_milestone_index(), rand_unreferenced_message());
        Insert::<(MilestoneIndex, UnreferencedMessage), ()>::insert(storage, &(index, unreferenced_message), &())
            .unwrap();
        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            &mut batch,
            &(index, unreferenced_message),
        )
        .unwrap();
    }

    let mut unreferenced_messages = HashMap::<MilestoneIndex, Vec<UnreferencedMessage>>::new();

    for _ in 0..5 {
        let index = rand_milestone_index();
        for _ in 0..5 {
            let unreferenced_message = rand_unreferenced_message();
            Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_insert(
                storage,
                &mut batch,
                &(index, unreferenced_message),
                &(),
            )
            .unwrap();
            unreferenced_messages
                .entry(index)
                .or_default()
                .push(unreferenced_message);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(MilestoneIndex, UnreferencedMessage), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, message_id), _) = result.unwrap();
        assert!(unreferenced_messages.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, unreferenced_messages.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, UnreferencedMessage), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(MilestoneIndex, UnreferencedMessage), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
