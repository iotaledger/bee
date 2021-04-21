// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{ address::Ed25519Address, output::OutputId};
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend::StorageBackend,
};
use bee_storage_rocksdb::{config::RocksDbConfigBuilder, storage::Storage};
use bee_test::rand::{address::rand_ed25519_address, output::rand_output_id};

use futures::stream::StreamExt;

use std::collections::HashMap;

const DB_DIRECTORY: &str = "./tests/database/ed25519_address_to_output_id";

#[tokio::test]
async fn ed25519_address_to_output_id_access() {
    let _ = std::fs::remove_dir_all(DB_DIRECTORY);

    let config = RocksDbConfigBuilder::default().with_path(DB_DIRECTORY.into()).finish();
    let storage = Storage::start(config).await.unwrap();

    let (address, output_id) = (rand_ed25519_address(), rand_output_id());

    assert!(
        !Exist::<(Ed25519Address, OutputId), ()>::exist(&storage, &(address, output_id))
            .await
            .unwrap()
    );
    assert!(Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&storage, &address)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(Ed25519Address, OutputId), ()>::insert(&storage, &(address, output_id), &())
        .await
        .unwrap();

    assert!(
        Exist::<(Ed25519Address, OutputId), ()>::exist(&storage, &(address, output_id))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&storage, &address)
            .await
            .unwrap()
            .unwrap(),
        vec![output_id]
    );

    Delete::<(Ed25519Address, OutputId), ()>::delete(&storage, &(address, output_id))
        .await
        .unwrap();

    assert!(
        !Exist::<(Ed25519Address, OutputId), ()>::exist(&storage, &(address, output_id))
            .await
            .unwrap()
    );
    assert!(Fetch::<Ed25519Address, Vec<OutputId>>::fetch(&storage, &address)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = Storage::batch_begin();

    for _ in 0usize..10usize {
        let (address, output_id) = (rand_ed25519_address(), rand_output_id());
        Insert::<(Ed25519Address, OutputId), ()>::insert(&storage, &(address, output_id), &())
            .await
            .unwrap();
        Batch::<(Ed25519Address, OutputId), ()>::batch_delete(&storage, &mut batch, &(address, output_id)).unwrap();
    }

    let mut output_ids = HashMap::<Ed25519Address, Vec<OutputId>>::new();

    for _ in 0usize..5usize {
        let address = rand_ed25519_address();
        for _ in 0usize..5usize {
            let output_id = rand_output_id();
            Batch::<(Ed25519Address, OutputId), ()>::batch_insert(&storage, &mut batch, &(address, output_id), &())
                .unwrap();
            output_ids.entry(address).or_default().push(output_id);
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(&storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some(((address, output_id), _)) = stream.next().await {
        assert!(output_ids.get(&address).unwrap().contains(&output_id));
        count += 1;
    }

    assert_eq!(count, output_ids.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(Ed25519Address, OutputId), ()>::truncate(&storage)
        .await
        .unwrap();

    let mut stream = AsStream::<(Ed25519Address, OutputId), ()>::stream(&storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());

    let _ = std::fs::remove_dir_all(DB_DIRECTORY);
}
