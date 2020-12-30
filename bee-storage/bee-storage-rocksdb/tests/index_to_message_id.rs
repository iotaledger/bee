// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::indexation::HashedIndex, MessageId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert},
    storage::Backend,
};
use bee_storage_rocksdb::{config::RocksDBConfigBuilder, storage::Storage};
use bee_test::rand::message::{rand_indexation, rand_message_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/index_to_message_id";

#[tokio::test]
async fn access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDBConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (index, message_id) = (rand_indexation().hash(), rand_message_id());

    assert!(
        !Exist::<(HashedIndex, MessageId), ()>::exist(&storage, &(index, message_id))
            .await
            .unwrap()
    );
    assert!(Fetch::<HashedIndex, Vec<MessageId>>::fetch(&storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(HashedIndex, MessageId), ()>::insert(&storage, &(index, message_id), &())
        .await
        .unwrap();

    assert!(
        Exist::<(HashedIndex, MessageId), ()>::exist(&storage, &(index, message_id))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<HashedIndex, Vec<MessageId>>::fetch(&storage, &index)
            .await
            .unwrap()
            .unwrap(),
        vec![message_id]
    );

    Delete::<(HashedIndex, MessageId), ()>::delete(&storage, &(index, message_id))
        .await
        .unwrap();

    assert!(
        !Exist::<(HashedIndex, MessageId), ()>::exist(&storage, &(index, message_id))
            .await
            .unwrap()
    );
    assert!(Fetch::<HashedIndex, Vec<MessageId>>::fetch(&storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (index, message_id) = (rand_indexation().hash(), rand_message_id());
        Insert::<(HashedIndex, MessageId), ()>::insert(&storage, &(index, message_id), &())
            .await
            .unwrap();
        Batch::<(HashedIndex, MessageId), ()>::batch_delete(&storage, &mut batch, &(index, message_id)).unwrap();
    }

    let mut message_ids = HashMap::new();

    for _ in 0usize..10usize {
        let (index, message_id) = (rand_indexation().hash(), rand_message_id());
        Batch::<(HashedIndex, MessageId), ()>::batch_insert(&storage, &mut batch, &(index, message_id), &()).unwrap();
        message_ids.insert(index, message_id);
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(HashedIndex, MessageId), ()>::stream(&storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some(((index, message_id), _)) = stream.next().await {
        assert_eq!(message_ids.get(&index).unwrap(), &message_id);
        count += 1;
    }

    assert_eq!(count, message_ids.len());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
