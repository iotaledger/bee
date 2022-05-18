// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::MilestoneIndex, MessageId};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, InsertStrict, MultiFetch, Truncate, Update},
    backend,
    backend::StorageBackendExt,
};
use bee_tangle::metadata::MessageMetadata;
use bee_test::rand::{message::rand_message_id, metadata::rand_message_metadata};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MessageId, MessageMetadata>
    + Fetch<MessageId, MessageMetadata>
    + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
    + InsertStrict<MessageId, MessageMetadata>
    + Delete<MessageId, MessageMetadata>
    + BatchBuilder
    + Batch<MessageId, MessageMetadata>
    + for<'a> AsIterator<'a, MessageId, MessageMetadata>
    + Truncate<MessageId, MessageMetadata>
    + Update<MessageId, MessageMetadata>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MessageId, MessageMetadata>
        + Fetch<MessageId, MessageMetadata>
        + for<'a> MultiFetch<'a, MessageId, MessageMetadata>
        + InsertStrict<MessageId, MessageMetadata>
        + Delete<MessageId, MessageMetadata>
        + BatchBuilder
        + Batch<MessageId, MessageMetadata>
        + for<'a> AsIterator<'a, MessageId, MessageMetadata>
        + Truncate<MessageId, MessageMetadata>
        + Update<MessageId, MessageMetadata>
{
}

pub fn message_id_to_metadata_access<B: StorageBackend>(storage: &B) {
    let (message_id, metadata) = (rand_message_id(), rand_message_metadata());

    assert!(!Exist::<MessageId, MessageMetadata>::exist_op(storage, &message_id).unwrap());
    assert!(
        storage
            .fetch::<MessageId, MessageMetadata>(&message_id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    InsertStrict::<MessageId, MessageMetadata>::insert_strict_op(storage, &message_id, &metadata).unwrap();
    assert!(Exist::<MessageId, MessageMetadata>::exist_op(storage, &message_id).unwrap());

    // calling `insert_strict` with the same `MessageId` but a different `MessageMetadata` should
    // not overwrite the old value.
    {
        let index = metadata.milestone_index().map_or(0, |i| *i + 1);
        let mut metadata = metadata.clone();
        metadata.set_milestone_index(MilestoneIndex(index));

        InsertStrict::<MessageId, MessageMetadata>::insert_strict_op(storage, &message_id, &metadata).unwrap();
    }
    assert_eq!(
        storage
            .fetch::<MessageId, MessageMetadata>(&message_id)
            .unwrap()
            .unwrap(),
        metadata,
        "`InsertStrict` should not overwrite"
    );

    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &metadata));

    let milestone_index = {
        let index = storage
            .fetch::<MessageId, MessageMetadata>(&message_id)
            .unwrap()
            .unwrap()
            .milestone_index();

        MilestoneIndex(index.map_or(0, |i| i.wrapping_add(1)))
    };

    Update::<MessageId, MessageMetadata>::update_op(storage, &message_id, |metadata: &mut MessageMetadata| {
        metadata.set_milestone_index(milestone_index);
    })
    .unwrap();

    assert_eq!(
        storage
            .fetch::<MessageId, MessageMetadata>(&message_id)
            .unwrap()
            .unwrap()
            .milestone_index(),
        Some(milestone_index),
    );

    Delete::<MessageId, MessageMetadata>::delete_op(storage, &message_id).unwrap();

    assert!(!Exist::<MessageId, MessageMetadata>::exist_op(storage, &message_id).unwrap());
    assert!(
        storage
            .fetch::<MessageId, MessageMetadata>(&message_id)
            .unwrap()
            .is_none()
    );

    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch_op(storage, &[message_id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut message_ids = Vec::new();
    let mut metadatas = Vec::new();

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        InsertStrict::<MessageId, MessageMetadata>::insert_strict_op(storage, &message_id, &metadata).unwrap();
        storage
            .batch_delete::<MessageId, MessageMetadata>(&mut batch, &message_id)
            .unwrap();
        message_ids.push(message_id);
        metadatas.push((message_id, None));
    }

    for _ in 0..10 {
        let (message_id, metadata) = (rand_message_id(), rand_message_metadata());
        storage
            .batch_insert::<MessageId, MessageMetadata>(&mut batch, &message_id, &metadata)
            .unwrap();
        message_ids.push(message_id);
        metadatas.push((message_id, Some(metadata)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MessageId, MessageMetadata>::iter_op(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (message_id, metadata) = result.unwrap();
        assert!(metadatas.contains(&(message_id, Some(metadata))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MessageId, MessageMetadata>::multi_fetch_op(storage, &message_ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), message_ids.len());

    for ((_, metadata), result) in metadatas.into_iter().zip(results.into_iter()) {
        assert_eq!(metadata, result.unwrap());
    }

    Truncate::<MessageId, MessageMetadata>::truncate_op(storage).unwrap();

    let mut iter = AsIterator::<MessageId, MessageMetadata>::iter_op(storage).unwrap();

    assert!(iter.next().is_none());
}
