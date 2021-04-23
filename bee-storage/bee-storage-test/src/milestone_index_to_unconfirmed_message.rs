// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_storage::{
    access::{AsStream, Batch, BatchBuilder, Delete, Exist, Fetch, Insert, Truncate},
    backend,
};
use bee_tangle::unconfirmed_message::UnconfirmedMessage;
use bee_test::rand::{milestone::rand_milestone_index, unconfirmed_message::rand_unconfirmed_message};

use futures::stream::StreamExt;

use std::collections::HashMap;

pub trait StorageBackend:
    backend::StorageBackend
    + Exist<(MilestoneIndex, UnconfirmedMessage), ()>
    + Fetch<MilestoneIndex, Vec<UnconfirmedMessage>>
    + Insert<(MilestoneIndex, UnconfirmedMessage), ()>
    + Delete<(MilestoneIndex, UnconfirmedMessage), ()>
    + BatchBuilder
    + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
    + for<'a> AsStream<'a, (MilestoneIndex, UnconfirmedMessage), ()>
    + Truncate<(MilestoneIndex, UnconfirmedMessage), ()>
{
}

impl<T> StorageBackend for T where
    T: backend::StorageBackend
        + Exist<(MilestoneIndex, UnconfirmedMessage), ()>
        + Fetch<MilestoneIndex, Vec<UnconfirmedMessage>>
        + Insert<(MilestoneIndex, UnconfirmedMessage), ()>
        + Delete<(MilestoneIndex, UnconfirmedMessage), ()>
        + BatchBuilder
        + Batch<(MilestoneIndex, UnconfirmedMessage), ()>
        + for<'a> AsStream<'a, (MilestoneIndex, UnconfirmedMessage), ()>
        + Truncate<(MilestoneIndex, UnconfirmedMessage), ()>
{
}

pub async fn milestone_index_to_unconfirmed_message_access<B: StorageBackend>(storage: &B) {
    let (index, unconfirmed_message) = (rand_milestone_index(), rand_unconfirmed_message());

    assert!(
        !Exist::<(MilestoneIndex, UnconfirmedMessage), ()>::exist(storage, &(index, unconfirmed_message))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    Insert::<(MilestoneIndex, UnconfirmedMessage), ()>::insert(storage, &(index, unconfirmed_message), &())
        .await
        .unwrap();

    assert!(
        Exist::<(MilestoneIndex, UnconfirmedMessage), ()>::exist(storage, &(index, unconfirmed_message))
            .await
            .unwrap()
    );
    assert_eq!(
        Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(storage, &index)
            .await
            .unwrap()
            .unwrap(),
        vec![unconfirmed_message]
    );

    Delete::<(MilestoneIndex, UnconfirmedMessage), ()>::delete(storage, &(index, unconfirmed_message))
        .await
        .unwrap();

    assert!(
        !Exist::<(MilestoneIndex, UnconfirmedMessage), ()>::exist(storage, &(index, unconfirmed_message))
            .await
            .unwrap()
    );
    assert!(Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(storage, &index)
        .await
        .unwrap()
        .unwrap()
        .is_empty());

    let mut batch = B::batch_begin();

    for _ in 0usize..10usize {
        let (index, unconfirmed_message) = (rand_milestone_index(), rand_unconfirmed_message());
        Insert::<(MilestoneIndex, UnconfirmedMessage), ()>::insert(storage, &(index, unconfirmed_message), &())
            .await
            .unwrap();
        Batch::<(MilestoneIndex, UnconfirmedMessage), ()>::batch_delete(
            storage,
            &mut batch,
            &(index, unconfirmed_message),
        )
        .unwrap();
    }

    let mut unconfirmed_messages = HashMap::<MilestoneIndex, Vec<UnconfirmedMessage>>::new();

    for _ in 0usize..5usize {
        let index = rand_milestone_index();
        for _ in 0usize..5usize {
            let unconfirmed_message = rand_unconfirmed_message();
            Batch::<(MilestoneIndex, UnconfirmedMessage), ()>::batch_insert(
                storage,
                &mut batch,
                &(index, unconfirmed_message),
                &(),
            )
            .unwrap();
            unconfirmed_messages.entry(index).or_default().push(unconfirmed_message);
        }
    }

    storage.batch_commit(batch, true).await.unwrap();

    let mut stream = AsStream::<(MilestoneIndex, UnconfirmedMessage), ()>::stream(storage)
        .await
        .unwrap();
    let mut count = 0;

    while let Some(((index, message_id), _)) = stream.next().await {
        assert!(unconfirmed_messages.get(&index).unwrap().contains(&message_id));
        count += 1;
    }

    assert_eq!(count, unconfirmed_messages.iter().fold(0, |acc, v| acc + v.1.len()));

    Truncate::<(MilestoneIndex, UnconfirmedMessage), ()>::truncate(storage)
        .await
        .unwrap();

    let mut stream = AsStream::<(MilestoneIndex, UnconfirmedMessage), ()>::stream(storage)
        .await
        .unwrap();

    assert!(stream.next().await.is_none());
}
