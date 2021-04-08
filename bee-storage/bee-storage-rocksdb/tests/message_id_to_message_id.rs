// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDbConfigBuilder, storage::Storage};
use bee_test::rand::message::rand_message_id;

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/message_id_to_message_id";

#[tokio::test]
async fn message_id_to_message_id_access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDbConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (parent, child) = (rand_message_id(), rand_message_id());

    assert!(
        !Exist::<(MessageId, MessageId), ()>::exist(&storage, &(parent, child))
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MessageId, Vec<MessageId>>::fetch(&storage, &parent)
            .await
            .unwrap()
            .unwrap()
            .is_empty()
    );

    Insert::<(MessageId, MessageId), ()>::insert(&storage, &(parent, child), &())
        .await
        .unwrap();

    assert!(
        Exist::<(MessageId, MessageId), ()>::exist(&storage, &(parent, child))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MessageId, Vec<MessageId>>::fetch(&storage, &parent)
            .await
            .unwrap()
            .unwrap(),
        vec![child]
    );

    Delete::<(MessageId, MessageId), ()>::delete(&storage, &(parent, child))
        .await
        .unwrap();

    assert!(
        !Exist::<(MessageId, MessageId), ()>::exist(&storage, &(parent, child))
            .await
            .unwrap()
    );
    assert!(
        Fetch::<MessageId, Vec<MessageId>>::fetch(&storage, &parent)
            .await
            .unwrap()
            .unwrap()
            .is_empty()
    );

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (parent, child) = (rand_message_id(), rand_message_id());
        Insert::<(MessageId, MessageId), ()>::insert(&storage, &(parent, child), &())
            .await
            .unwrap();
        Batch::<(MessageId, MessageId), ()>::batch_delete(&storage, &mut batch, &(parent, child)).unwrap();
    }

    let mut edges = HashMap::new();

    for _ in 0usize..10usize {
        let (parent, child) = (rand_message_id(), rand_message_id());
        Batch::<(MessageId, MessageId), ()>::batch_insert(&storage, &mut batch, &(parent, child), &()).unwrap();
        edges.insert(parent, child);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(MessageId, MessageId), ()>::stream(&storage).await.unwrap();
    let mut count = 0;

    while let Some(((parent, child), _)) = stream.next().await {
        assert_eq!(edges.get(&parent).unwrap(), &child);
        count += 1;
    }

    assert_eq!(count, edges.len());

    Truncate::<(MessageId, MessageId), ()>::truncate(&storage)
        .await
        .unwrap();

    let mut stream = AsStream::<(MessageId, MessageId), ()>::stream(&storage).await.unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
