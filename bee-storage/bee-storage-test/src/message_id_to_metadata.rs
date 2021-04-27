// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::MessageId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::metadata::MessageMetadata;
use bee_test::rand::{message::rand_message_id, metadata::rand_message_metadata};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, MessageMetadata>
    + Fetch<MessageId, MessageMetadata>
    + Insert<MessageId, MessageMetadata>
    + Delete<MessageId, MessageMetadata>
    + BatchBuilder
    + Batch<MessageId, MessageMetadata>
    + for<'a> AsStream<'a, MessageId, MessageMetadata>
    + Truncate<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MessageId, MessageMetadata>
        + Fetch<MessageId, MessageMetadata>
        + Insert<MessageId, MessageMetadata>
        + Delete<MessageId, MessageMetadata>
        + BatchBuilder
        + Batch<MessageId, MessageMetadata>
        + for<'a> AsStream<'a, MessageId, MessageMetadata>
        + Truncate<MessageId, MessageMetadata>
{
}

pub async fn message_id_to_metadata_access<B: StorageBackend>(storage: &B) {
    let (message_id, metadata) = (rand_message_id(), rand_message_metadata());

    assert!(
        !Exist::<MessageId, MessageMetadata>::exist(storage, &message_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .await
            .unwrap()
            .is_none()
    );

    Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &metadata)
        .await
        .unwrap();

    assert!(
        Exist::<MessageId, MessageMetadata>::exist(storage, &message_id)
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        metadata.pack_new()
    );

    Delete::<MessageId, MessageMetadata>::delete(storage, &message_id)
        .await
        .unwrap();

    assert!(
        !Exist::<MessageId, MessageMetadata>::exist(storage, &message_id)
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .await
            .unwrap()
            .is_none()
    );

    let mut batch = B::batch_begin();

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &metadata)
            .await
            .unwrap();
        Batch::<MessageId, MessageMetadata>::batch_delete(storage, &mut batch, &message_id).unwrap();
    }

    let mut metadatas = HashMap::new();

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        Batch::<MessageId, MessageMetadata>::batch_insert(storage, &mut batch, &message_id, &metadata).unwrap();
        metadatas.insert(message_id, metadata);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MessageId, MessageMetadata>::stream(storage).await.unwrap();
    let mut count = 0;

    while let Some((message_id, metadata)) = stream.next().await {
        assert_eq!(metadatas.get(&message_id).unwrap().pack_new(), metadata.pack_new());
        count += 1;
    }

    assert_eq!(count, metadatas.len());

    Truncate::<MessageId, MessageMetadata>::truncate(storage).await.unwrap();

    let mut stream = AsStream::<MessageId, MessageMetadata>::stream(storage).await.unwrap();

    assert!(stream.next().await.is_none());
}
