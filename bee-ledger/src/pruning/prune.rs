// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{collect::*, error::Error};

use crate::consensus::{event::PrunedIndex, StorageBackend};

use bee_message::{
    milestone::Milestone,
    payload::Payload,
    prelude::{HashedIndex, MilestoneIndex},
    Message, MessageId,
};

use bee_runtime::event::Bus;
use bee_storage::access::{Batch, Fetch};
use bee_tangle::{
    metadata::MessageMetadata, ms_tangle::StorageHooks, unconfirmed_message::UnconfirmedMessage, MsTangle,
};
use log::{debug, info};

pub async fn prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    bus: &Bus<'_>,
    target_index: MilestoneIndex,
) -> Result<(), Error> {
    info!("Pruning...");

    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;
    debug_assert!(target_index > start_index);

    // Get access to the storage backend of the Tangle.
    let storage = tangle.hooks();

    for index in *start_index..=*target_index {
        // Collect the data that can be safely pruned.
        let (confirmed, edges, new_seps, indexations) = collect_confirmed_data(tangle, index).await?;
        let (unconfirmed, unconfirmed_edges, unconfirmed_indexations) =
            collect_unconfirmed_data(storage, index.into()).await?;

        debug!(
            "New entry point index {}. (Selected {} new solid entry points).",
            index,
            new_seps.len()
        );

        // Replace SEPs in the Tangle.
        tangle.replace_solid_entry_points(new_seps).await;

        // Remember up to which index we determined SEPs.
        tangle.update_entry_point_index(index.into());

        // Prepare a batch of ...
        let mut batch = B::batch_begin();

        // ... the confirmed data,
        let mut num_messages = prune_messages(storage, &mut batch, confirmed).await?;
        let mut num_edges = prune_edges(storage, &mut batch, edges).await?;
        let mut num_indexations = prune_indexations(storage, &mut batch, indexations).await?;

        // ... and the unconfirmed data,
        num_messages += prune_unconfirmed_messages(storage, &mut batch, index.into(), unconfirmed).await?;
        num_edges += prune_edges(storage, &mut batch, unconfirmed_edges).await?;
        num_indexations += prune_indexations(storage, &mut batch, unconfirmed_indexations).await?;

        // ... and the milestone data itself.
        prune_milestone(storage, &mut batch, index.into()).await?;

        storage
            .batch_commit(batch, true)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        debug!(
            "Pruned milestone {} consisting of {} messages, {} edges, {} indexations.",
            index, num_messages, num_edges, num_indexations,
        );

        bus.dispatch(PrunedIndex(index.into()));
    }

    tangle.update_pruning_index(target_index);

    debug!("Pruning index now at {}.", target_index);

    Ok(())
}

async fn prune_messages<B: StorageBackend, M: IntoIterator<Item = MessageId>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    messages: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for message_id in messages.into_iter() {
        // "&StorageHooks(ResourceHandle(B))": *** => B
        Batch::<MessageId, Message>::batch_delete(&***storage, batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        Batch::<MessageId, MessageMetadata>::batch_delete(&***storage, batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_indexations<B: StorageBackend, I: IntoIterator<Item = (HashedIndex, MessageId)>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    indexes: I,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (index, message_id) in indexes.into_iter() {
        Batch::<(HashedIndex, MessageId), ()>::batch_delete(&***storage, batch, &(index, message_id))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_edges<B: StorageBackend, E: IntoIterator<Item = Edge>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    edges: E,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (from, to) in edges.into_iter().map(|edge| (edge.from, edge.to)) {
        Batch::<(MessageId, MessageId), ()>::batch_delete(&***storage, batch, &(from, to))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_unconfirmed_messages<B: StorageBackend, M: IntoIterator<Item = UnconfirmedMessage>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    index: MilestoneIndex,
    messages: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for unconfirmed_message in messages.into_iter() {
        Batch::<(MilestoneIndex, UnconfirmedMessage), ()>::batch_delete(
            &***storage,
            batch,
            &(index, unconfirmed_message),
        )
        .map_err(|e| Error::StorageError(Box::new(e)))?;

        // Message
        Batch::<MessageId, Message>::batch_delete(&***storage, batch, &unconfirmed_message.message_id())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        // MessageMetadata
        Batch::<MessageId, MessageMetadata>::batch_delete(&***storage, batch, &unconfirmed_message.message_id())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_milestone<B: StorageBackend>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    Batch::<MilestoneIndex, Milestone>::batch_delete(&***storage, batch, &index)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    Ok(())
}
