// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::message::{rand_message, rand_message_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/message_id_to_message";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (message_id, message) = (rand_message_id(), rand_message());

    assert!(!Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    Insert::<MessageId, Message>::insert(&storage, &message_id, &message)
        .await
        .unwrap();

    assert!(Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());
    assert_eq!(
        Fetch::<MessageId, Message>::fetch(&storage, &message_id)
            .await
            .unwrap()
            .unwrap()
            .pack_new(),
        message.pack_new()
    );

    Delete::<MessageId, Message>::delete(&storage, &message_id)
        .await
        .unwrap();

    assert!(!Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (message_id, message) = (rand_message_id(), rand_message());
        Insert::<MessageId, Message>::insert(&storage, &message_id, &message)
            .await
            .unwrap();
        Batch::<MessageId, Message>::batch_delete(&storage, &mut batch, &message_id).unwrap();
    }

    let mut messages = HashMap::new();

    for _ in 0usize..10usize {
        let (message_id, message) = (rand_message_id(), rand_message());
        Batch::<MessageId, Message>::batch_insert(&storage, &mut batch, &message_id, &message).unwrap();
        messages.insert(message_id, message);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<MessageId, Message>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some((message_id, message)) = stream.next().await {
        assert_eq!(messages.get(&message_id).unwrap().pack_new(), message.pack_new());
        count += 1;
    }

    assert_eq!(count, messages.len());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
