// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};
use bee_tangle::metadata::MessageMetadata;
use bee_test::rand::{message::rand_message_id, metadata::rand_message_metadata};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, MessageMetadata>
    + Fetch<MessageId, MessageMetadata>
    + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
    + Insert<MessageId, MessageMetadata>
    + Delete<MessageId, MessageMetadata>
    + BatchBuilder
    + Batch<MessageId, MessageMetadata>
    + for<'a> AsIterator<'a, MessageId, MessageMetadata>
    + Truncate<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MessageId, MessageMetadata>
        + Fetch<MessageId, MessageMetadata>
        + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
        + Insert<MessageId, MessageMetadata>
        + Delete<MessageId, MessageMetadata>
        + BatchBuilder
        + Batch<MessageId, MessageMetadata>
        + for<'a> AsIterator<'a, MessageId, MessageMetadata>
        + Truncate<MessageId, MessageMetadata>
{
}

pub fn message_id_to_metadata_access<B: StorageBackend>(storage: &B) {
    let (message_id, metadata) = (rand_message_id(), rand_message_metadata());

    assert!(!Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &metadata).unwrap();

    assert!(Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert_eq!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .unwrap(),
        metadata
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &metadata));

    Delete::<MessageId, MessageMetadata>::delete(storage, &message_id).unwrap();

    assert!(!Exist::<MessageId, MessageMetadata>::exist(storage, &message_id).unwrap());
    assert!(
        Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut message_ids = Vec::new();
    let mut metadatas = Vec::new();

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        Insert::<MessageId, MessageMetadata>::insert(storage, &message_id, &metadata).unwrap();
        Batch::<MessageId, MessageMetadata>::batch_delete(storage, &mut batch, &message_id).unwrap();
        message_ids.push(message_id);
        metadatas.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        Batch::<MessageId, MessageMetadata>::batch_insert(storage, &mut batch, &message_id, &metadata).unwrap();
        message_ids.push(message_id);
        metadatas.push((message_id, Some(metadata)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MessageId, MessageMetadata>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (message_id, metadata) = result.unwrap();
        assert!(metadatas.contains(&(message_id, Some(metadata))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch(storage, &message_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), message_ids.len());

    for ((_, metadata), result) in metadatas.into_iter().zip(results.into_iter()) {
        assert_eq!(metadata, result.unwrap());
    }

    Truncate::<MessageId, MessageMetadata>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MessageId, MessageMetadata>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
