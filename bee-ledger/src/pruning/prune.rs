// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{collect::*, error::Error};

use crate::consensus::StorageBackend;

use bee_message::{
    milestone::Milestone,
    payload::Payload,
    prelude::{HashedIndex, MilestoneIndex},
    Message, MessageId,
};

use bee_storage::access::{Batch, Fetch};
use bee_tangle::{
    metadata::MessageMetadata, ms_tangle::StorageHooks, unconfirmed_message::UnconfirmedMessage, MsTangle,
};
use log::{debug, info};

pub async fn prune<B: StorageBackend>(tangle: &MsTangle<B>, target_index: MilestoneIndex) -> Result<(), Error> {
    info!("Pruning database...");

    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;

    // Collect the data that can be safely pruned.
    let (confirmed, edges, new_seps, indexes) = collect_confirmed_data(tangle, start_index, target_index).await?;

    debug!("Determined {} new solid entry points.", new_seps.len());

    // Replace SEPs in the Tangle.
    tangle.replace_solid_entry_points(new_seps).await;

    // Remember up to which index we determined SEPs.
    tangle.update_entry_point_index(target_index);

    // Get access to the storage backend of the Tangle.
    let storage = tangle.hooks();

    debug!(
        "Pruning {} confirmed messages, {} edges, {} indexation payloads.",
        confirmed.len(),
        edges.len(),
        indexes.len()
    );

    prune_messages(storage, confirmed).await?;
    prune_indexes(storage, indexes).await?;
    prune_edges(storage, edges).await?;
    prune_milestones(storage, start_index, target_index).await?;
    prune_unconfirmed(storage, start_index, target_index).await?;

    tangle.update_pruning_index(target_index);

    debug!("Pruning index now at {}.", target_index);

    Ok(())
}

async fn prune_messages<B: StorageBackend, M: IntoIterator<Item = MessageId>>(
    storage: &StorageHooks<B>,
    messages: M,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();
    let mut num_pruned = 0;

    for message_id in messages.into_iter() {
        // "&StorageHooks(ResourceHandle(B))": *** => B
        Batch::<MessageId, Message>::batch_delete(&***storage, &mut batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        Batch::<MessageId, MessageMetadata>::batch_delete(&***storage, &mut batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    debug!("Pruned {} messages.", num_pruned);

    Ok(())
}

async fn prune_indexes<B: StorageBackend, I: IntoIterator<Item = (HashedIndex, MessageId)>>(
    storage: &StorageHooks<B>,
    indexes: I,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();
    let mut num_pruned = 0;

    for (index, message_id) in indexes.into_iter() {
        Batch::<(HashedIndex, MessageId), ()>::batch_delete(&***storage, &mut batch, &(index, message_id))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    debug!("Pruned {} indexes.", num_pruned);

    Ok(())
}

async fn prune_edges<B: StorageBackend, E: IntoIterator<Item = Edge>>(
    storage: &StorageHooks<B>,
    edges: E,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();
    let mut num_pruned = 0;

    for (from, to) in edges.into_iter().map(|edge| (edge.from, edge.to)) {
        Batch::<(MessageId, MessageId), ()>::batch_delete(&***storage, &mut batch, &(from, to))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    debug!("Pruned {} edges.", num_pruned);

    Ok(())
}

async fn prune_milestones<B: StorageBackend>(
    storage: &StorageHooks<B>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();
    let mut num_pruned = 0;

    for milestone_index in *start_index..=*target_index {
        Batch::<MilestoneIndex, Milestone>::batch_delete(&***storage, &mut batch, &milestone_index.into())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    debug!("Pruned {} milestones.", num_pruned);

    Ok(())
}

async fn prune_unconfirmed<B: StorageBackend>(
    storage: &StorageHooks<B>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(), Error> {
    let mut batch = B::batch_begin();
    let mut unconfirmed = Vec::default();
    let mut indexes = Vec::default();
    let mut edges = Vec::default();
    let mut num_pruned = 0;

    for milestone_index in *start_index..=*target_index {
        // Get the unconfirmed/unreferenced messages.
        let unconfirmed_messages =
            Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(&***storage, &milestone_index.into())
                .await
                .map_err(|e| Error::StorageError(Box::new(e)))?
                .unwrap();

        if unconfirmed_messages.is_empty() {
            continue;
        }

        for unconfirmed_message in unconfirmed_messages {
            Batch::<(MilestoneIndex, UnconfirmedMessage), ()>::batch_delete(
                &***storage,
                &mut batch,
                &(milestone_index.into(), unconfirmed_message),
            )
            .map_err(|e| Error::StorageError(Box::new(e)))?;

            let (payload, parents) = Fetch::<MessageId, Message>::fetch(&***storage, &unconfirmed_message.message_id())
                .await
                .map_err(|e| Error::StorageError(Box::new(e)))?
                .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
                // TODO: explain why that `unwrap` is safe? Why do we know that the `Fetch` must find that message?
                .unwrap();

            unconfirmed.push(*unconfirmed_message.message_id());

            // Handle indexation payload
            if let Some(Payload::Indexation(payload)) = payload {
                let index = payload.hash();
                let message_id = *unconfirmed_message.message_id();
                indexes.push((index, message_id));
            }

            // Edges of unconfirmed messages
            for parent in parents.iter() {
                edges.push(Edge {
                    from: *parent,
                    to: *unconfirmed_message.message_id(),
                });
            }

            num_pruned += 1;
        }
    }

    // Prune the (MilestoneIndex, UnconfirmedMessage) pairs
    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    debug!("Pruned {} unconfirmed messages.", num_pruned);

    prune_messages(storage, unconfirmed).await?;
    prune_indexes(storage, indexes).await?;
    prune_edges(storage, edges).await?;

    Ok(())
}
