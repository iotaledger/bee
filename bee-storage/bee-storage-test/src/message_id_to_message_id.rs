// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::MessageId;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::message::rand_message_id;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MessageId, MessageId), ()>
    + Fetch<MessageId, Vec<MessageId>>
    + Insert<(MessageId, MessageId), ()>
    + Delete<(MessageId, MessageId), ()>
    + BatchBuilder
    + Batch<(MessageId, MessageId), ()>
    + for<'a> AsIterator<'a, (MessageId, MessageId), ()>
    + Truncate<(MessageId, MessageId), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MessageId, MessageId), ()>
        + Fetch<MessageId, Vec<MessageId>>
        + Insert<(MessageId, MessageId), ()>
        + Delete<(MessageId, MessageId), ()>
        + BatchBuilder
        + Batch<(MessageId, MessageId), ()>
        + for<'a> AsIterator<'a, (MessageId, MessageId), ()>
        + Truncate<(MessageId, MessageId), ()>
{
}

pub fn message_id_to_message_id_access<B: StorageBackend>(storage: &B) {
    let (parent, child) = (rand_message_id(), rand_message_id());

    assert!(!Exist::<(MessageId, MessageId), ()>::exist(storage, &(parent, child)).unwrap());
    assert!(
        storage
            .fetch::<MessageId, Vec<MessageId>>(&parent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(MessageId, MessageId), ()>::insert(storage, &(parent, child), &()).unwrap();

    assert!(Exist::<(MessageId, MessageId), ()>::exist(storage, &(parent, child)).unwrap());
    assert_eq!(
        storage.fetch::<MessageId, Vec<MessageId>>(&parent).unwrap().unwrap(),
        vec![child]
    );

    Delete::<(MessageId, MessageId), ()>::delete(storage, &(parent, child)).unwrap();

    assert!(!Exist::<(MessageId, MessageId), ()>::exist(storage, &(parent, child)).unwrap());
    assert!(
        storage
            .fetch::<MessageId, Vec<MessageId>>(&parent)
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (parent, child) = (rand_message_id(), rand_message_id());
        Insert::<(MessageId, MessageId), ()>::insert(storage, &(parent, child), &()).unwrap();
        Batch::<(MessageId, MessageId), ()>::batch_delete(storage, &mut batch, &(parent, child)).unwrap();
    }

    let mut edges = HashMap::<MessageId, Vec<MessageId>>::new();

    for _ in 0..5 {
        let parent = rand_message_id();
        for _ in 0..5 {
            let child = rand_message_id();
            Batch::<(MessageId, MessageId), ()>::batch_insert(storage, &mut batch, &(parent, child), &()).unwrap();
            edges.entry(parent).or_default().push(child);
        }
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<(MessageId, MessageId), ()>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let ((parent, child), _) = result.unwrap();
        assert!(edges.get(&parent).unwrap().contains(&child));
        count += 1;
    }

    assert_eq!(count, edges.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MessageId, MessageId), ()>::truncate(storage).unwrap();

    let mut iter = AsIterator::<(MessageId, MessageId), ()>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
