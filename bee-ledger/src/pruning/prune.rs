// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{collect::*, error::Error, PruningConfig};

use crate::{
    consensus::{event::PrunedIndex, StorageBackend},
    types::OutputDiff,
};

use bee_message::{
    milestone::Milestone,
    prelude::{HashedIndex, MilestoneIndex},
    Message, MessageId,
};

use bee_runtime::event::Bus;
use bee_storage::access::Batch;
use bee_tangle::{
    metadata::MessageMetadata, ms_tangle::StorageHooks, unconfirmed_message::UnconfirmedMessage, MsTangle,
};
use log::{debug, info};

pub async fn prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    bus: &Bus<'_>,
    target_index: MilestoneIndex,
    config: &PruningConfig,
) -> Result<(), Error> {
    info!("Pruning...");

    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;
    debug_assert!(target_index > start_index);

    // Batch size
    // let batch_size = config.batch_size() as u32;
    // let mut batch_idx = 1;

    // Get access to the storage backend of the Tangle.
    let storage = tangle.hooks();

    for index in *start_index..=*target_index {
        let index: MilestoneIndex = index.into();

        // Collect the data that can be safely pruned.
        let (confirmed, edges, new_seps, indexations) = collect_confirmed_data(tangle, index).await?;
        let (unconfirmed, unconfirmed_edges, unconfirmed_indexations) =
            collect_unconfirmed_data(storage, index).await?;

        debug!(
            "New entry point index {}. (Selected {} new solid entry points).",
            index,
            new_seps.len()
        );

        // Replace SEPs in the Tangle.
        tangle.replace_solid_entry_points(new_seps).await;

        // Remember up to which index we determined SEPs.
        tangle.update_entry_point_index(index);

        // Prepare a batch of ...
        let mut batch = B::batch_begin();

        // ... the confirmed data,
        let mut num_messages = prune_messages(storage, &mut batch, confirmed).await?;
        let mut num_edges = prune_edges(storage, &mut batch, edges).await?;
        let mut num_indexations = prune_indexations(storage, &mut batch, indexations).await?;

        // ... and the unconfirmed data,
        num_messages += prune_unconfirmed_messages(storage, &mut batch, index, unconfirmed).await?;
        num_edges += prune_edges(storage, &mut batch, unconfirmed_edges).await?;
        num_indexations += prune_indexations(storage, &mut batch, unconfirmed_indexations).await?;

        // ... and the milestone data itself.
        prune_milestone(storage, &mut batch, index).await?;
        // prune_receipt(storage, &mut batch, index).await?;
        prune_output_diff(storage, &mut batch, index).await?;

        // if batch_idx % batch_size == 0 {
        storage
            .batch_commit(batch, true)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        //     is_dirty = false;
        // } else {
        //     is_dirty = true;
        // }

        // batch_idx += 1;

        debug!(
            "Pruned milestone {} consisting of {} messages, {} edges, {} indexations.",
            index, num_messages, num_edges, num_indexations,
        );

        tangle.update_pruning_index(index);
        debug!("Pruning index now at {}.", index);
    }

    // if is_dirty {
    //     storage
    //         .batch_commit(batch, true)
    //         .await
    //         .map_err(|e| Error::StorageError(Box::new(e)))?;
    // }

    bus.dispatch(PrunedIndex(target_index));

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

async fn prune_output_diff<B: StorageBackend>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    Batch::<MilestoneIndex, OutputDiff>::batch_delete(&***storage, batch, &index)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    Ok(())
}

// async fn prune_receipt<B: StorageBackend>(
//     storage: &StorageHooks<B>,
//     batch: &mut B::Batch,
//     index: MilestoneIndex,
// ) -> Result<(), Error> {
//     Batch::<MilestoneIndex, Receipt>::batch_delete(&***storage, batch, &index)
//         .map_err(|e| Error::StorageError(Box::new(e)))?;

//     Ok(())
// }
