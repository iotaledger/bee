// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{Message, MessageId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::message::{random_message, random_message_id};

use futures::stream::StreamExt;

#[tokio::test]
async fn access() {
    let config = RocksDBConfigBuilder::default().finish();
    let storage = Storage::start(config).await.unwrap();

    let message_id = random_message_id();
    let message_1 = random_message();

    assert!(!Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    storage.insert(&message_id, &message_1).await.unwrap();

    assert!(Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());

    let message_2 = Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(message_1.pack_new(), message_2.pack_new());

    Delete::<MessageId, Message>::delete(&storage, &message_id)
        .await
        .unwrap();

    assert!(!Exist::<MessageId, Message>::exist(&storage, &message_id).await.unwrap());
    assert!(Fetch::<MessageId, Message>::fetch(&storage, &message_id)
        .await
        .unwrap()
        .is_none());

    let mut message_ids = Vec::new();

    for _ in 0usize..100usize {
        let (message_id, message) = (random_message_id(), random_message());
        message_ids.push(message_id);
        storage.insert(&message_id, &message).await.unwrap();
    }

    let mut batch = Storage::batch_begin();

    for (i, message_id) in message_ids.iter().enumerate() {
        storage
            .batch_insert(&mut batch, &random_message_id(), &random_message())
            .unwrap();
        if i % 2 == 0 {
            Batch::<MessageId, Message>::batch_delete(&storage, &mut batch, message_id).unwrap();
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    for (i, message_id) in message_ids.iter().enumerate() {
        if i % 2 == 0 {
            assert!(!Exist::<MessageId, Message>::exist(&storage, message_id).await.unwrap());
        } else {
            assert!(Exist::<MessageId, Message>::exist(&storage, message_id).await.unwrap());
        }
    }

    let mut stream = AsStream::<MessageId, Message>::stream(&storage).await.unwrap();

    while let Some((key, value)) = stream.next().await {
        println!("{:?} {:?}", key, value);
    }
}
