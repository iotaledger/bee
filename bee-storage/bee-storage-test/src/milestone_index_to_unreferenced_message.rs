// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_tangle::unreferenced_message::UnreferencedMessage;
use bee_test::rand::{milestone::rand_milestone_index, unreferenced_message::rand_unreferenced_message};

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
        !storage
            .exist::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<MilestoneIndex, Vec<UnreferencedMessage>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    storage
        .insert::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message), &())
        .unwrap();

    assert!(
        storage
            .exist::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message))
            .unwrap()
    );
    assert_eq!(
        storage
            .fetch::<MilestoneIndex, Vec<UnreferencedMessage>>(&index)
            .unwrap()
            .unwrap(),
        vec![unreferenced_message]
    );

    storage
        .delete::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message))
        .unwrap();

    assert!(
        !storage
            .exist::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<MilestoneIndex, Vec<UnreferencedMessage>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, unreferenced_message) = (rand_milestone_index(), rand_unreferenced_message());
        storage
            .insert::<(MilestoneIndex, UnreferencedMessage), ()>(&(index, unreferenced_message), &())
            .unwrap();
        storage
            .batch_delete::<(MilestoneIndex, UnreferencedMessage), ()>(&mut batch, &(index, unreferenced_message))
            .unwrap();
    }

    let mut unreferenced_messages = HashMap::<MilestoneIndex, Vec<UnreferencedMessage>>::new();

    for _ in 0..5 {
        let index = rand_milestone_index();
        for _ in 0..5 {
            let unreferenced_message = rand_unreferenced_message();
            storage
                .batch_insert::<(MilestoneIndex, UnreferencedMessage), ()>(
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

    let iter = storage.iter::<(MilestoneIndex, UnreferencedMessage), ()>().unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, message_id), _) = result.unwrap();
        assert!(unreferenced_messages.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, unreferenced_messages.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, UnreferencedMessage), ()>::truncate_op(storage).unwrap();

    let mut iter = storage.iter::<(MilestoneIndex, UnreferencedMessage), ()>().unwrap();

    assert!(iter.next().is_none());
}
