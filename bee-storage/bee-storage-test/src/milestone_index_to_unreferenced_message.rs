// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::unreferenced_message::UnreferencedMessage;
use bee_test::rand::{milestone::rand_milestone_index, unreferenced_message::rand_unreferenced_message};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MilestoneIndex, UnreferencedMessage), ()>
    + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
    + Insert<(MilestoneIndex, UnreferencedMessage), ()>
    + Delete<(MilestoneIndex, UnreferencedMessage), ()>
    + BatchBuilder
    + Batch<(MilestoneIndex, UnreferencedMessage), ()>
    + for<'a> AsStream<'a, (MilestoneIndex, UnreferencedMessage), ()>
    + Truncate<(MilestoneIndex, UnreferencedMessage), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MilestoneIndex, UnreferencedMessage), ()>
        + Fetch<MilestoneIndex, Vec<UnreferencedMessage>>
        + Insert<(MilestoneIndex, UnreferencedMessage), ()>
        + Delete<(MilestoneIndex, UnreferencedMessage), ()>
        + BatchBuilder
        + Batch<(MilestoneIndex, UnreferencedMessage), ()>
        + for<'a> AsStream<'a, (MilestoneIndex, UnreferencedMessage), ()>
        + Truncate<(MilestoneIndex, UnreferencedMessage), ()>
{
}

pub async fn milestone_index_to_unreferenced_message_access<B: StorageBackend>(storage: &B) {
    let (index, unreferenced_message) = (rand_milestone_index(), rand_unreferenced_message());

    assert!(
        !Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(MilestoneIndex, UnreferencedMessage), ()>::insert(storage, &(index, unreferenced_message), &())
        .await
        .unwrap();

    assert!(
        Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
            .await
            .unwrap()
            .unwrap(),
        vec![unreferenced_message]
    );

    Delete::<(MilestoneIndex, UnreferencedMessage), ()>::delete(storage, &(index, unreferenced_message))
        .await
        .unwrap();

    assert!(
        !Exist::<(MilestoneIndex, UnreferencedMessage), ()>::exist(storage, &(index, unreferenced_message))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (index, unreferenced_message) = (rand_milestone_index(), rand_unreferenced_message());
        Insert::<(MilestoneIndex, UnreferencedMessage), ()>::insert(storage, &(index, unreferenced_message), &())
            .await
            .unwrap();
        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            &mut batch,
            &(index, unreferenced_message),
        )
        .unwrap();
    }

    let mut unreferenced_messages = HashMap::<MilestoneIndex, Vec<UnreferencedMessage>>::new();

    for _ in 0usize..5usize {
        let index = rand_milestone_index();
        for _ in 0usize..5usize {
            let unreferenced_message = rand_unreferenced_message();
            Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_insert(
                storage,
                &mut batch,
                &(index, unreferenced_message),
                &(),
            )
            .unwrap();
            unreferenced_messages.entry(index).or_default().push(unreferenced_message);
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(MilestoneIndex, UnreferencedMessage), ()>::stream(storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some(((index, message_id), _)) = stream.next().await {
        assert!(unreferenced_messages.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, unreferenced_messages.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, UnreferencedMessage), ()>::truncate(storage)
        .await
        .unwrap();

    let mut stream = AsStream::<(MilestoneIndex, UnreferencedMessage), ()>::stream(storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());
}
