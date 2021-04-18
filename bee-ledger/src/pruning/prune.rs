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

use log::{error, info};
use once_cell::sync::OnceCell;

use std::{collections::HashSet, sync::Mutex};

// **NOTE**: In order to ensure that everything that entered the database gets pruned eventually the following mechanism
// is used:
// (1) Incoming messages immediatedly get assigned the LMI (current latest milestone index known by the node)
// (2) Most of them will get confirmed eventually, and are referencable by a particular CMI (the closest future
// milestone that references them)
// However, that means that for a given message X LMI(X) <= CMI(X) may not be true. That is certainly the case for
// requested messages, that are required to solidify an older milestone. When the message comes in it gets assigned the
// current LMI, while its confirming milestone might be several milestone indexes ago already. So that would mean that
// when we try to prune LMI(X) (as unconfirmed) it would fail, because it would have been pruned already at CMI(X).
//
// The other way of failure is also possible: A message X was pruned at LMI(X), but when CMI(X) is about to pruned it
// can't be used to fully traverse the past-cone of that particular milestone, and hence will make it impossible to
// determine new SEP set.
//
// CASE A: CMI < LMI (requested message)        => pruning at CMI problematic for unconfirmed messages
//      Solution: just ignore the failure, and assume that the message has been pruned already at CMI, because it was
// confirmed CASE B: CMI > LMI (regular gossip message)   => pruning at LMI problematic
//      Solution: use `ADDITIONAL_DELAY` such that CMI < LMI + UNCONFIRMED_PRUNING_DELAY_FACTOR
//

/// Determines how many BMDs we delay pruning of still unconfirmed messages. It basically determines the range to be:
/// [LMI(X)..LMI(X)+BMD*UNCONFIRMED_PRUNING_DELAY_FACTOR] within which a message *MUST* be confirmed, i.e. contains
/// CMI(X), so it basically relies on the Coordinator node to not confirm a message anytime later than that, otherwise
/// traversing the about-to-be-pruned past-cone of a milestone to determine the new SEP set would fail.
const UNCONFIRMED_PRUNING_DELAY_FACTOR: u32 = 2; // Pruning (if still unconf.) happens 60 milestones after its LMI(X)

fn confirmed_removal_list() -> &'static Mutex<HashSet<MessageId>> {
    static INSTANCE: OnceCell<Mutex<HashSet<MessageId>>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let m = HashSet::default();
        Mutex::new(m)
    })
}

fn unconfirmed_removal_list() -> &'static Mutex<HashSet<MessageId>> {
    static INSTANCE: OnceCell<Mutex<HashSet<MessageId>>> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let m = HashSet::default();
        Mutex::new(m)
    })
}

pub async fn prune<B: StorageBackend>(
    tangle: &MsTangle<B>,
    bus: &Bus<'_>,
    target_index: MilestoneIndex,
    config: &PruningConfig,
    below_max_depth: u32,
) -> Result<(), Error> {
    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;
    assert!(target_index >= start_index);

    // If `start_index == 1` (lowest possible start index), we need to deactivate "unconfirmed" pruning.
    let unconfirmed_additional_pruning_delay = below_max_depth * UNCONFIRMED_PRUNING_DELAY_FACTOR;
    let unconfirmed_start_index = if *start_index >= unconfirmed_additional_pruning_delay {
        Some(start_index - unconfirmed_additional_pruning_delay)
    } else {
        None
    };

    // Get access to the storage backend of the Tangle.
    let storage = tangle.hooks();

    info!("Pruning database until milestone {}...", target_index);

    // Prepare a batch of delete operations on the database.
    let mut batch = B::batch_begin();

    // We get us a clone of the current SEP set. We are the only ones that make changes to that tangle state, so we can
    // be sure it can't be invalidated in the meantime while we do the past-cone traversal.
    let old_seps = tangle.get_all_solid_entry_points().await;
    dbg!(old_seps.len());

    // Collect the data that can be safely pruned. In order to find the correct set of SEPs during this pruning we need
    // to walk the past-cone from the `target_index` backwards, and not step-by-step from `start_index` to
    // `target_index` as this would require additional logic to remove redundant SEPs again. If memory or performance
    // becomes an issue, reduce the `pruning_interval` in the config.
    let (confirmed, edges, mut new_seps, indexations) =
        collect_confirmed_data(tangle, &storage, target_index, &old_seps).await?;

    // TEMP: make sure, that the collect process doesn't yield duplicate message ids.
    {
        let mut removal_list = confirmed_removal_list().lock().unwrap();
        for confirmed_id in &confirmed {
            if removal_list.contains(confirmed_id) {
                error!("double removal of a confirmed message");
            } else {
                removal_list.insert(*confirmed_id);
            }
        }
        // println!("len(confirmed_removal_list) = {}", removal_list.len());
        dbg!(removal_list.len());
        drop(removal_list);
    }

    // // TEMPORARILY CHECK ALL NEW SEPS
    // // We can still do this as long as they're not pruned.
    // let mut num_relevant_seps = 0;
    // for sep_id in new_seps.iter().map(|(sep, _)| sep.message_id()) {
    //     let sep_approvers = tangle.get_children(sep_id).await.unwrap();
    //     'inner: for sep_approver in &sep_approvers {
    //         // Only SEPs are considered relevant that:
    //         // (a) still have an unconfirmed approver, or
    //         // (b) are referenced by *directly* by confirmed messages of the following ms beyond `target_index`
    //         match tangle.get_metadata(sep_approver).await.unwrap().milestone_index() {
    //             None => {
    //                 num_relevant_seps += 1;
    //                 break 'inner;
    //             }
    //             // Some(ms_index) if ms_index == target_index + 1 => {
    //             //     num_relevant_seps += 1;
    //             //     break 'inner;
    //             // }
    //             Some(_) => {
    //                 num_relevant_seps += 1;
    //                 break 'inner;

    //             }
    //         }
    //     }
    // }
    // dbg!(new_seps.len(), num_relevant_seps);

    // Move all young enough old SEPs to the new set
    for (old_sep, index) in old_seps {
        // Keep the old SEP as long as there might be unconfirmed messages referencing it.
        if index + unconfirmed_additional_pruning_delay > target_index {
            new_seps.insert(old_sep, index);
        }
    }

    let num_new_seps = dbg!(new_seps.len());
    tangle.replace_solid_entry_points(new_seps).await;
    // for (sep, index) in new_seps.drain() {
    //     tangle.add_solid_entry_point(sep, index).await;
    // }

    // Remember up to which index we determined SEPs.
    tangle.update_entry_point_index(target_index);
    info!(
        "Entry point index now at {}. (Selected {} new solid entry points).",
        target_index, num_new_seps,
    );

    // Add the confirmed data to the batch.
    let num_messages = prune_messages(storage, &mut batch, confirmed).await?;
    let num_edges = prune_edges(storage, &mut batch, edges).await?;
    let num_indexations = prune_indexations(storage, &mut batch, indexations).await?;

    let mut num_unconf_messages = 0;
    let mut num_unconf_edges = 0;
    let mut num_unconf_indexations = 0;

    // Handling of remaining still unconfirmed messages
    if let Some(unconfirmed_start_index) = unconfirmed_start_index {
        let unconfirmed_target_index = target_index - unconfirmed_additional_pruning_delay;
        info!(
            "Pruning unconfirmed messages until milestone {}...",
            unconfirmed_target_index
        );
        assert!(unconfirmed_target_index < target_index);

        let (still_unconfirmed, unconfirmed_edges, unconfirmed_indexations) =
            collect_still_unconfirmed_data(storage, unconfirmed_start_index, unconfirmed_target_index).await?;

        {
            // TEMPORARILY ADD TO REMOVAL LIST
            let mut removal_list = unconfirmed_removal_list().lock().unwrap();
            for unconfirmed_id in still_unconfirmed.iter().map(|(_, b)| b.message_id()) {
                if removal_list.contains(unconfirmed_id) {
                    error!("double removal of a unconfirmed message");
                } else {
                    removal_list.insert(*unconfirmed_id);
                }
            }
            // println!("len(unconfirmed_removal_list) = {}", removal_list.len());
            dbg!(removal_list.len());
            drop(removal_list);
        }

        // Add the unconfirmed data to the batch.
        num_unconf_messages += prune_received_messages(storage, &mut batch, still_unconfirmed).await?;
        num_unconf_edges += prune_edges(storage, &mut batch, unconfirmed_edges).await?;
        num_unconf_indexations += prune_indexations(storage, &mut batch, unconfirmed_indexations).await?;
    }

    // Add milestone related data to the batch.
    let num_milestones = prune_milestones(storage, &mut batch, start_index, target_index).await?;
    let num_output_diffs = prune_output_diffs(storage, &mut batch, start_index, target_index).await?;

    let mut num_receipts = 0;
    // Add receipts optionally.
    if config.prune_receipts() {
        let receipts = collect_receipts(storage, start_index, target_index).await?;
        num_receipts += prune_receipts(storage, &mut batch, receipts).await?;
    }

    storage
        .batch_commit(batch, true)
        .await
        // If that error actually happens we set the database to 'corrupted'!
        .map_err(|e| Error::BatchCommitError(Box::new(e)))?;

    info!(
        "PRUNING SUMMARY: \n{} milestones, {}/{} un/confirmed messages, {}/{} un/confirmed edges, {}/{} un/confirmed indexations, {} output_diffs, {} receipts",
        num_milestones,
        num_unconf_messages,
        num_messages,
        num_unconf_edges,
        num_edges,
        num_unconf_indexations,
        num_indexations,
        num_output_diffs,
        num_receipts,
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

async fn prune_received_messages<B: StorageBackend, M: IntoIterator<Item = (MilestoneIndex, UnconfirmedMessage)>>(
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
