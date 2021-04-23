// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_test::rand::message::{rand_message, rand_message_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, Message>
    + Fetch<MessageId, Message>
    + Insert<MessageId, Message>
    + Delete<MessageId, Message>
    + BatchBuilder
    + Batch<MessageId, Message>
    + for<'a> AsStream<'a, MessageId, Message>
    + Truncate<MessageId, Message>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MessageId, Message>
        + Fetch<MessageId, Message>
        + Insert<MessageId, Message>
        + Delete<MessageId, Message>
        + BatchBuilder
        + Batch<MessageId, Message>
        + for<'a> AsStream<'a, MessageId, Message>
        + Truncate<MessageId, Message>
{
}

pub async fn message_id_to_message_access<B: StorageBackend>(storage: &B) {
    let (message_id, message) = (rand_message_id(), rand_message());

    assert!(!Exist::<MessageId, Message>::exist(storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(storage, &message_id)
        .await
        .unwrap()
        .is_none());

    Insert::<MessageId, Message>::insert(storage, &message_id, &message)
        .await
        .unwrap();

    assert!(Exist::<MessageId, Message>::exist(storage, &message_id).await.unwrap());
    assert_eq!(
        Fetch::<MessageId, Message>::fetch(storage, &message_id)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        message.pack_new()
    );

    Delete::<MessageId, Message>::delete(storage, &message_id)
        .await
        .unwrap();

    assert!(!Exist::<MessageId, Message>::exist(storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(storage, &message_id)
        .await
        .unwrap()
        .is_none());

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (message_id, message) = (rand_message_id(), rand_message());
        Insert::<MessageId, Message>::insert(storage, &message_id, &message)
            .await
            .unwrap();
        Batch::<MessageId, Message>::batch_delete(storage, &mut batch, &message_id).unwrap();
    }

    let mut messages = HashMap::new();

    for _ in 0usize..10usize {
        let (message_id, message) = (rand_message_id(), rand_message());
        Batch::<MessageId, Message>::batch_insert(storage, &mut batch, &message_id, &message).unwrap();
        messages.insert(message_id, message);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MessageId, Message>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((message_id, message)) = stream.next().await {
        assert_eq!(messages.get(&message_id).unwrap().pack_new(), message.pack_new());
        count += 1;
    }

    assert_eq!(count, messages.len());

    Truncate::<MessageId, Message>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<MessageId, Message>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
