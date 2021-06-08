// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_test::rand::message::{rand_message, rand_message_id};

use futures::stream::StreamExt;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, Message>
    + Fetch<MessageId, Message>
    + MultiFetch<MessageId, Message>
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
        + MultiFetch<MessageId, Message>
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
    let results = MultiFetch::<MessageId, Message>::multi_fetch(storage, &[message_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MessageId, Message>::insert(storage, &message_id, &message)
        .await
        .unwrap();

    assert!(Exist::<MessageId, Message>::exist(storage, &message_id).await.unwrap());
    assert_eq!(
        Fetch::<MessageId, Message>::fetch(storage, &message_id)
            .await
            .unwrap()
            .unwrap(),
        message
    );
    let results = MultiFetch::<MessageId, Message>::multi_fetch(storage, &[message_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &message));

    Delete::<MessageId, Message>::delete(storage, &message_id)
        .await
        .unwrap();

    assert!(!Exist::<MessageId, Message>::exist(storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(storage, &message_id)
        .await
        .unwrap()
        .is_none());
    let results = MultiFetch::<MessageId, Message>::multi_fetch(storage, &[message_id])
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut message_ids = Vec::new();
    let mut messages = Vec::new();

    for _ in 0..10 {
        let (message_id, message) = (rand_message_id(), rand_message());
        Insert::<MessageId, Message>::insert(storage, &message_id, &message)
            .await
            .unwrap();
        Batch::<MessageId, Message>::batch_delete(storage, &mut batch, &message_id).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, message) = (rand_message_id(), rand_message());
        Batch::<MessageId, Message>::batch_insert(storage, &mut batch, &message_id, &message).unwrap();
        message_ids.push(message_id);
        messages.push((message_id, Some(message)));
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MessageId, Message>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let (message_id, message) = result.unwrap();
        assert!(messages.contains(&(message_id, Some(message))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MessageId, Message>::multi_fetch(storage, &message_ids)
        .await
        .unwrap();

    assert_eq!(results.len(), message_ids.len());

    for ((_, message), result) in messages.into_iter().zip(results.into_iter()) {
        assert_eq!(message, result.unwrap());
    }

    Truncate::<MessageId, Message>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<MessageId, Message>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
