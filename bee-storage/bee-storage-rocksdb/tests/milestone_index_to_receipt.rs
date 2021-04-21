// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::types::Receipt;
use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDbConfigBuilder, storage::Storage};
use bee_test::rand::{milestone::rand_milestone_index, receipt::rand_receipt};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/milestone_index_to_receipt";

#[tokio::test]
async fn milestone_index_to_receipt_access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDbConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (index, receipt) = (rand_milestone_index(), rand_receipt());

    assert!(
        !Exist::<(MilestoneIndex, Receipt), ()>::exist(&storage, &(index, receipt.clone()))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(MilestoneIndex, Receipt), ()>::insert(&storage, &(index, receipt.clone()), &())
        .await
        .unwrap();

    assert!(
        Exist::<(MilestoneIndex, Receipt), ()>::exist(&storage, &(index, receipt.clone()))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&storage, &index)
            .await
            .unwrap()
            .unwrap(),
        vec![receipt.clone()]
    );

    Delete::<(MilestoneIndex, Receipt), ()>::delete(&storage, &(index, receipt.clone()))
        .await
        .unwrap();

    assert!(
        !Exist::<(MilestoneIndex, Receipt), ()>::exist(&storage, &(index, receipt.clone()))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (index, receipt) = (rand_milestone_index(), rand_receipt());
        Insert::<(MilestoneIndex, Receipt), ()>::insert(&storage, &(index, receipt.clone()), &())
            .await
            .unwrap();
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(&storage, &mut batch, &(index, receipt)).unwrap();
    }

    let mut receipts = HashMap::<MilestoneIndex, Vec<Receipt>>::new();

    for _ in 0usize..5usize {
        let index = rand_milestone_index();
        for _ in 0usize..5usize {
            let receipt = rand_receipt();
            Batch::<(MilestoneIndex, Receipt), ()>::batch_insert(&storage, &mut batch, &(index, receipt.clone()), &())
                .unwrap();
            receipts.entry(index).or_default().push(receipt);
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(MilestoneIndex, Receipt), ()>::stream(&storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some(((index, message_id), _)) = stream.next().await {
        assert!(receipts.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, receipts.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, Receipt), ()>::truncate(&storage)
        .await
        .unwrap();

    let mut stream = AsStream::<(MilestoneIndex, Receipt), ()>::stream(&storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
