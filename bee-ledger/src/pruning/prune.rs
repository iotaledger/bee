// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{collect::*, error::Error, PruningConfig};

use crate::{
    consensus::{event::PrunedIndex, StorageBackend},
    types::{OutputDiff, Receipt},
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

    // Get access to the storage backend of the Tangle.
    let storage = tangle.hooks();

    // Collect the data that can be safely pruned. In order to find the correct set of SEPs during this pruning we need
    // to walk the past-cone from the `target_index` backwards, and not step-by-step from `start_index` to
    // `target_index` as this would require additional logic to remove redundant SEPs again. If memory or performance
    // becomes an issue, reduce the `pruning_interval` in the config.
    let (confirmed, edges, new_seps, indexations) = collect_confirmed_data(tangle, target_index).await?;
    let (unconfirmed, unconfirmed_edges, unconfirmed_indexations) =
        collect_unconfirmed_data(storage, start_index, target_index).await?;

    // TEMPORARLY CHECK ALL SEPS
    let mut num_relevant_seps = 0;
    for sep_id in new_seps.iter().map(|(sep, _)| sep.message_id()) {
        let sep_approvers = tangle.get_children(sep_id).await.unwrap();
        'inner: for approver in &sep_approvers {
            // only SEPs are relevant that still have an unconfirmed approver
            match tangle.get_metadata(approver).await.unwrap().milestone_index() {
                None => {
                    num_relevant_seps += 1;
                    break 'inner;
                }
                Some(ms_index) if ms_index == target_index + 1 => {
                    num_relevant_seps += 1;
                    break 'inner;
                }
                Some(_) => {}
            }
        }
    }
    info!(
        "Collected {}, but only {} are relevant.",
        new_seps.len(),
        num_relevant_seps
    );

    // Prepare a batch of delete operations on the database.
    let mut batch = B::batch_begin();

    // Add the confirmed data to the batch.
    let mut num_messages = prune_messages(storage, &mut batch, confirmed).await?;
    let mut num_edges = prune_edges(storage, &mut batch, edges).await?;
    let mut num_indexations = prune_indexations(storage, &mut batch, indexations).await?;

    // Add the unconfirmed data to the batch.
    num_messages += prune_unconfirmed_messages(storage, &mut batch, unconfirmed).await?;
    num_edges += prune_edges(storage, &mut batch, unconfirmed_edges).await?;
    num_indexations += prune_indexations(storage, &mut batch, unconfirmed_indexations).await?;

    // Add milestone related data to the batch.
    let num_milestones = prune_milestones(storage, &mut batch, start_index, target_index).await?;
    let num_output_diffs = prune_output_diffs(storage, &mut batch, start_index, target_index).await?;

    // Add receipts optionally.
    if config.prune_receipts() {
        let receipts = collect_receipts(storage, start_index, target_index).await?;
        prune_receipts(storage, &mut batch, receipts).await?;
    }

    storage
        .batch_commit(batch, true)
        .await
        // If that error actually happens we set the database to 'corrupted'!
        .map_err(|e| Error::BatchCommitError(Box::new(e)))?;

    info!("Milestones from {} to {} have been pruned.", start_index, target_index);

    debug!(
        "{} milestones, {} messages, {} edges, {} indexations, {} output_diffs have been successfully pruned.",
        num_milestones, num_messages, num_edges, num_indexations, num_output_diffs
    );

    // Replace SEPs in the Tangle. We do this **AFTER** we commited the batch and it returned no error. This way we
    // allow the database to be reset to its previous state, and repeat the pruning.
    let num_new_seps = new_seps.len();
    tangle.replace_solid_entry_points(new_seps).await;

    // Remember up to which index we determined SEPs.
    tangle.update_entry_point_index(target_index);

    info!(
        "Entry point index now at {}. (Selected {} new solid entry points).",
        target_index, num_new_seps,
    );

    tangle.update_pruning_index(target_index);
    info!("Pruning index now at {}.", target_index);

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

    for (from, to) in edges.into_iter().map(|edge| (edge.from_parent, edge.to_child)) {
        Batch::<(MessageId, MessageId), ()>::batch_delete(&***storage, batch, &(from, to))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_unconfirmed_messages<B: StorageBackend, M: IntoIterator<Item = (MilestoneIndex, UnconfirmedMessage)>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    messages: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (index, unconfirmed_message) in messages.into_iter() {
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

async fn prune_milestones<B: StorageBackend>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, Milestone>::batch_delete(&***storage, batch, &index.into())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_output_diffs<B: StorageBackend>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, OutputDiff>::batch_delete(&***storage, batch, &index.into())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }
    Ok(num_pruned)
}

async fn prune_receipts<B: StorageBackend, R: IntoIterator<Item = (MilestoneIndex, Receipt)>>(
    storage: &StorageHooks<B>,
    batch: &mut B::Batch,
    receipts: R,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (index, receipt) in receipts.into_iter() {
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(&***storage, batch, &(index, receipt))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}
