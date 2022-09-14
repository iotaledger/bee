// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    payload::milestone::{MilestoneId, MilestonePayload},
    protocol::protocol_parameters,
    rand::{milestone::rand_milestone_id, payload::rand_milestone_payload},
};
use bee_storage::{
    access::{AsIterator, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, MultiFetch, Truncate},
    backend,
};

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<MilestoneId, MilestonePayload>
    + Fetch<MilestoneId, MilestonePayload>
    + for<'a> MultiFetch<'a, MilestoneId, MilestonePayload>
    + Insert<MilestoneId, MilestonePayload>
    + Delete<MilestoneId, MilestonePayload>
    + BatchBuilder
    + Batch<MilestoneId, MilestonePayload>
    + for<'a> AsIterator<'a, MilestoneId, MilestonePayload>
    + Truncate<MilestoneId, MilestonePayload>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<MilestoneId, MilestonePayload>
        + Fetch<MilestoneId, MilestonePayload>
        + for<'a> MultiFetch<'a, MilestoneId, MilestonePayload>
        + Insert<MilestoneId, MilestonePayload>
        + Delete<MilestoneId, MilestonePayload>
        + BatchBuilder
        + Batch<MilestoneId, MilestonePayload>
        + for<'a> AsIterator<'a, MilestoneId, MilestonePayload>
        + Truncate<MilestoneId, MilestonePayload>
{
}

pub fn milestone_id_to_milestone_payload_access<B: StorageBackend>(storage: &B) {
    let protocol_parameters = protocol_parameters();
    let (id, payload) = (
        rand_milestone_id(),
        rand_milestone_payload(protocol_parameters.protocol_version()),
    );

    assert!(!Exist::<MilestoneId, MilestonePayload>::exist(storage, &id).unwrap());
    assert!(
        Fetch::<MilestoneId, MilestonePayload>::fetch(storage, &id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneId, MilestonePayload>::multi_fetch(storage, &[id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    Insert::<MilestoneId, MilestonePayload>::insert(storage, &id, &payload).unwrap();

    assert!(Exist::<MilestoneId, MilestonePayload>::exist(storage, &id).unwrap());
    assert_eq!(
        Fetch::<MilestoneId, MilestonePayload>::fetch(storage, &id)
            .unwrap()
            .unwrap(),
        payload
    );
    let results = MultiFetch::<MilestoneId, MilestonePayload>::multi_fetch(storage, &[id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(Some(v))) if v == &payload));

    Delete::<MilestoneId, MilestonePayload>::delete(storage, &id).unwrap();

    assert!(!Exist::<MilestoneId, MilestonePayload>::exist(storage, &id).unwrap());
    assert!(
        Fetch::<MilestoneId, MilestonePayload>::fetch(storage, &id)
            .unwrap()
            .is_none()
    );
    let results = MultiFetch::<MilestoneId, MilestonePayload>::multi_fetch(storage, &[id])
        .unwrap()
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 1);
    assert!(matches!(results.get(0), Some(Ok(None))));

    let mut batch = B::batch_begin();
    let mut ids = Vec::new();
    let mut payloads = Vec::new();

    for _ in 0..10 {
        let (id, payload) = (
            rand_milestone_id(),
            rand_milestone_payload(protocol_parameters.protocol_version()),
        );
        Insert::<MilestoneId, MilestonePayload>::insert(storage, &id, &payload).unwrap();
        Batch::<MilestoneId, MilestonePayload>::batch_delete(storage, &mut batch, &id).unwrap();
        ids.push(id);
        payloads.push((id, None));
    }

    for _ in 0..10 {
        let (id, payload) = (
            rand_milestone_id(),
            rand_milestone_payload(protocol_parameters.protocol_version()),
        );
        Batch::<MilestoneId, MilestonePayload>::batch_insert(storage, &mut batch, &id, &payload).unwrap();
        ids.push(id);
        payloads.push((id, Some(payload)));
    }

    storage.batch_commit(batch, true).unwrap();

    let iter = AsIterator::<MilestoneId, MilestonePayload>::iter(storage).unwrap();
    let mut count = 0;

    for result in iter {
        let (id, payload) = result.unwrap();
        assert!(payloads.contains(&(id, Some(payload))));
        count += 1;
    }

    assert_eq!(count, 10);

    let results = MultiFetch::<MilestoneId, MilestonePayload>::multi_fetch(storage, &ids)
        .unwrap()
        .collect::<Vec<_>>();

    assert_eq!(results.len(), ids.len());

    for ((_, payload), result) in payloads.into_iter().zip(results.into_iter()) {
        assert_eq!(payload, result.unwrap());
    }

    Truncate::<MilestoneId, MilestonePayload>::truncate(storage).unwrap();

    let mut iter = AsIterator::<MilestoneId, MilestonePayload>::iter(storage).unwrap();

    assert!(iter.next().is_none());
}
