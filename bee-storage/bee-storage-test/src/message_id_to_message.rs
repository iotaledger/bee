// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
    backend::StorageBackendExt,
};
use bee_test::rand::message::{rand_message, rand_message_id};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, Message>
    + Fetch<MessageId, Message>
    + for<'a> MultiFetch<'a, MessageId, Message>
    + Insert<MessageId, Message>
    + Delete<MessageId, Message>
    + BatchBuilder
    + Batch<MessageId, Message>
    + for<'a> AsIterator<'a, MessageId, Message>
    + Truncate<MessageId, Message>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MessageId, Message>
        + Fetch<MessageId, Message>
        + for<'a> MultiFetch<'a, MessageId, Message>
        + Insert<MessageId, Message>
        + Delete<MessageId, Message>
        + BatchBuilder
        + Batch<MessageId, Message>
        + for<'a> AsIterator<'a, MessageId, Message>
        + Truncate<MessageId, Message>
{
}

pub fn message_id_to_message_access<B: StorageBackend>(storage: &B) {
    let (message_id, message) = (rand_message_id(), rand_message());

    assert!(!Exist::<MessageId, Message>::exist_op(storage, &message_id).unwrap());
    assert!(storage.fetch::<MessageId, Message>(&message_id).unwrap().is_none());
    let results = MultiFetch::<MessageId, Message>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MessageId, Message>::insert_op(storage, &message_id, &message).unwrap();

    let message = rand_message();
    Insert::<MessageId, Message>::insert_op(storage, &message_id, &message).unwrap();
    assert_eq!(
        storage.fetch::<MessageId, Message>(&message_id).unwrap().as_ref(),
        Some(&message),
        "insert should overwrite"
    );

    assert!(Exist::<MessageId, Message>::exist_op(storage, &message_id).unwrap());
    assert_eq!(
        storage.fetch::<MessageId, Message>(&message_id).unwrap().unwrap(),
        message
    );
    let results = MultiFetch::<MessageId, Message>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &message));

    Delete::<MessageId, Message>::delete_op(storage, &message_id).unwrap();

    assert!(!Exist::<MessageId, Message>::exist_op(storage, &message_id).unwrap());
    assert!(storage.fetch::<MessageId, Message>(&message_id).unwrap().is_none());
    let results = MultiFetch::<MessageId, Message>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut message_ids = Vec::new();
    let mut messages = Vec::new();

    for _ in 0..10 {
        let (message_id, message) = (rand_message_id(), rand_message());
        Insert::<MessageId, Message>::insert_op(storage, &message_id, &message).unwrap();
        Batch::<MessageId, Message>::batch_delete_op(storage, &mut batch, &message_id).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, message) = (rand_message_id(), rand_message());
        Batch::<MessageId, Message>::batch_insert_op(storage, &mut batch, &message_id, &message).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, Some(message)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MessageId, Message>::iter_op(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (message_id, message) = result.unwrap();
        assert!(messages.contains(&(message_id, Some(message))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MessageId, Message>::multi_fetch_op(storage, &message_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), message_ids.len());

    for ((_, message), result) in messages.into_iter().zip(results.into_iter()) {
        assert_eq!(message, result.unwrap());
    }

    Truncate::<MessageId, Message>::truncate_op(storage).unwrap();

    let mut iter = AsIterator::<MessageId, Message>::iter_op(storage).unwrap();

    assert!(iter.next().is_none());
}
