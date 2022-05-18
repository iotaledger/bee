// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::{payload::indexation::PaddedIndex, MessageId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::{message::rand_message_id, payload::rand_indexation_payload};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(PaddedIndex, MessageId), ()>
    + Fetch<PaddedIndex, Vec<MessageId>>
    + Insert<(PaddedIndex, MessageId), ()>
    + Delete<(PaddedIndex, MessageId), ()>
    + BatchBuilder
    + Batch<(PaddedIndex, MessageId), ()>
    + for<'a> AsIterator<'a, (PaddedIndex, MessageId), ()>
    + Truncate<(PaddedIndex, MessageId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(PaddedIndex, MessageId), ()>
        + Fetch<PaddedIndex, Vec<MessageId>>
        + Insert<(PaddedIndex, MessageId), ()>
        + Delete<(PaddedIndex, MessageId), ()>
        + BatchBuilder
        + Batch<(PaddedIndex, MessageId), ()>
        + for<'a> AsIterator<'a, (PaddedIndex, MessageId), ()>
        + Truncate<(PaddedIndex, MessageId), ()>
{
}

pub fn index_to_message_id_access<B: StorageBackend>(storage: &B) {
    let (index, message_id) = (rand_indexation_payload().padded_index(), rand_message_id());

    assert!(
        !storage
            .exist::<(PaddedIndex, MessageId), ()>(&(index, message_id))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<PaddedIndex, Vec<MessageId>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(PaddedIndex, MessageId), ()>::insert_op(storage, &(index, message_id), &()).unwrap();

    assert!(
        storage
            .exist::<(PaddedIndex, MessageId), ()>(&(index, message_id))
            .unwrap()
    );
    assert_eq!(
        storage.fetch::<PaddedIndex, Vec<MessageId>>(&index).unwrap().unwrap(),
        vec![message_id]
    );

    storage
        .delete::<(PaddedIndex, MessageId), ()>(&(index, message_id))
        .unwrap();

    assert!(
        !storage
            .exist::<(PaddedIndex, MessageId), ()>(&(index, message_id))
            .unwrap()
    );
    assert!(
        storage
            .fetch::<PaddedIndex, Vec<MessageId>>(&index)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (index, message_id) = (rand_indexation_payload().padded_index(), rand_message_id());
        Insert::<(PaddedIndex, MessageId), ()>::insert_op(storage, &(index, message_id), &()).unwrap();
        storage
            .batch_delete::<(PaddedIndex, MessageId), ()>(&mut batch, &(index, message_id))
            .unwrap();
    }

    let mut message_ids = HashMap::<PaddedIndex, Vec<MessageId>>::new();

    for _ in 0..5 {
        let index = rand_indexation_payload().padded_index();
        for _ in 0..5 {
            let message_id = rand_message_id();
            storage
                .batch_insert::<(PaddedIndex, MessageId), ()>(&mut batch, &(index, message_id), &())
                .unwrap();
            message_ids.entry(index).or_default().push(message_id);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(PaddedIndex, MessageId), ()>::iter_op(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((index, message_id), _) = result.unwrap();
        assert!(message_ids.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, message_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(PaddedIndex, MessageId), ()>::truncate_op(storage).unwrap();

    let mut iter = AsIterator::<(PaddedIndex, MessageId), ()>::iter_op(storage).unwrap();

    assert!(iter.next().is_none());
}
