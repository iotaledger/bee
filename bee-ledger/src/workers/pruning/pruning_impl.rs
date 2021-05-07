// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{collect::*, config::PruningConfig, error::Error};

use crate::{
    types::{OutputDiff, Receipt},
    workers::{event::PrunedIndex, storage::StorageBackend},
};

use bee_message::{
    milestone::Milestone,
    prelude::{MilestoneIndex, PaddedIndex},
    Message, MessageId,
};

use bee_runtime::event::Bus;
use bee_storage::access::Batch;
use bee_tangle::{metadata::MessageMetadata, unreferenced_message::UnreferencedMessage as ReceivedMessageId, MsTangle};

use log::{error, info};
use once_cell::sync::OnceCell;

use std::{collections::HashMap, sync::Mutex};

// **NB**:
// In order to ensure that everything that entered the database gets pruned eventually the following mechanism
// is applied:
//
// (1) Incoming messages immediatedly get assigned the LMI (current latest milestone index known by the node)
// (2) Most of them will get confirmed eventually, and are referencable by a particular CMI (the closest future
// milestone that references them)
//
// However, that means that for a given message X LMI(X) <= CMI(X) may not be true. That is certainly the case for
// requested messages, that are required to solidify an older milestone. When the message comes in it gets assigned the
// current LMI, while its confirming milestone might be several milestone indexes ago already. So that would mean that
// when we try to prune LMI(X) (as unconfirmed) it would fail, because it would have been pruned already at CMI(X).
//
// The other way of failure is also possible: A message X was pruned at LMI(X), but when CMI(X) is about to pruned it
// can't be used to fully traverse the past-cone of that particular milestone, and hence will make it impossible to
// determine the new SEP set.
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
///
/// Pruning (if still unconf.) happens x-BMD milestones after its LMI(X)

const UNCONFIRMED_PRUNING_DELAY_FACTOR: u32 = 4;

pub async fn prune<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    bus: &Bus<'_>,
    pruning_target_index: MilestoneIndex,
    config: &PruningConfig,
    below_max_depth: u32,
) -> Result<(), Error> {
    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;
    assert!(pruning_target_index >= start_index);

    // If `start_index == 1` (lowest possible start index), we need to deactivate "unconfirmed" pruning.
    let unconfirmed_additional_pruning_delay = below_max_depth * UNCONFIRMED_PRUNING_DELAY_FACTOR;
    let unconfirmed_start_index = if *start_index >= unconfirmed_additional_pruning_delay {
        Some(start_index - unconfirmed_additional_pruning_delay)
    } else {
        None
    };

    info!("Pruning database until milestone {}...", pruning_target_index);

    // Prepare a batch of delete operations on the database.
    let mut batch = S::batch_begin();

    // We get us a clone of the current SEP set. We are the only ones that make changes to that particular tangle state,
    // so we can be sure it can't be invalidated in the meantime while we do the past-cone traversal to find the new
    // set.
    let old_seps = tangle.get_all_solid_entry_points().await;
    let num_old_seps = old_seps.len();

    // Collect the data that can be safely pruned. In order to find the correct set of SEPs during this pruning we need
    // to walk the past-cone from the `target_index` backwards, and not step-by-step from `start_index` to
    // `target_index` as this would require additional logic to remove redundant SEPs again. If memory or performance
    // becomes an issue, reduce the `pruning_interval` in the config.
    let (confirmed, edges, collected_seps, indexations) =
        collect_confirmed_data(tangle, &storage, pruning_target_index, &old_seps).await?;
    let num_collected_seps = collected_seps.len();

    // TEMP: make sure, that the collect process doesn't yield duplicate message ids.
    // **NOTE**: For some reason the enclosing block is necessary in async function despet the manual drop.
    {
        let mut removal_list = debugging::unique_confirmed_checklist().lock().unwrap();
        for confirmed_id in &confirmed {
            if let Some(index) = removal_list.get(confirmed_id) {
                error!(
                    "Collected confirmed message {} twice. First added during {}, now at {}. This is a bug!",
                    confirmed_id, index, pruning_target_index
                );
            } else {
                removal_list.insert(*confirmed_id, pruning_target_index);
            }
        }
        // dbg!(removal_list.len());
        drop(removal_list); // just here to be explicit about it
    }

    // We can still do this as long as they're not pruned.
    let mut new_seps = Seps::default();
    for (sep, index) in collected_seps.into_iter().map(|(sep, index)| (sep, index)) {
        let sep_approvers = tangle.get_children(sep.message_id()).await.unwrap();
        'inner: for sep_approver in &sep_approvers {
            if matches!(
                tangle.get_metadata(sep_approver).await.unwrap().milestone_index(),
                Some(milestone_index) if milestone_index > pruning_target_index
            ) {
                new_seps.insert(sep, index);
                break 'inner;
            }
        }
    }
    let num_new_seps = new_seps.len();

    dbg!(num_old_seps, num_collected_seps, num_new_seps);

    tangle.replace_solid_entry_points(new_seps).await;

    // Remember up to which index we determined SEPs.
    tangle.update_entry_point_index(pruning_target_index);
    info!(
        "Entry point index now at {}. (Selected {} new solid entry points).",
        pruning_target_index, num_new_seps,
    );

    // Add the confirmed data to the batch.
    let num_messages = prune_messages(storage, &mut batch, confirmed).await?;
    let num_edges = prune_edges(storage, &mut batch, edges).await?;
    let num_indexations = prune_indexations(storage, &mut batch, indexations).await?;

    let mut num_unconf_messages = 0;
    let mut num_unconf_edges = 0;
    let mut num_unconf_indexations = 0;
    let mut num_received_ids = 0;

    // Handling of remaining still unconfirmed messages
    if let Some(unconfirmed_start_index) = unconfirmed_start_index {
        let unconfirmed_target_index = pruning_target_index - unconfirmed_additional_pruning_delay;

        assert!(unconfirmed_target_index < pruning_target_index);
        assert!(unconfirmed_target_index >= unconfirmed_start_index);

        info!(
            "Pruning unconfirmed messages until milestone {}...",
            unconfirmed_target_index
        );

        let (received, unconfirmed_messages, unconfirmed_edges, unconfirmed_indexations) =
            collect_still_unconfirmed_data(storage, unconfirmed_start_index, unconfirmed_target_index).await?;

        // TEMPORARILY ADD TO REMOVAL LIST
        {
            let mut removal_list = debugging::unique_unconfirmed_checklist().lock().unwrap();
            for unconfirmed_id in received.iter().map(|(_, b)| b.message_id()) {
                if let Some(index) = removal_list.get(unconfirmed_id) {
                    error!(
                        "Collected UNconfirmed message {} twice. First added during {}, now at {}. This is a bug!",
                        unconfirmed_id, index, pruning_target_index
                    );
                } else {
                    removal_list.insert(*unconfirmed_id, pruning_target_index);
                }
            }
            // dbg!(removal_list.len());
            drop(removal_list); // just here to be explicit about it
        }

        // Add the unconfirmed data to the batch.
        num_unconf_messages = prune_messages(storage, &mut batch, unconfirmed_messages).await?;
        num_unconf_edges = prune_edges(storage, &mut batch, unconfirmed_edges).await?;
        num_unconf_indexations = prune_indexations(storage, &mut batch, unconfirmed_indexations).await?;

        num_received_ids = prune_received_ids(storage, &mut batch, received).await?;
    }

    // Add milestone related data to the batch.
    let num_milestones = prune_milestones(storage, &mut batch, start_index, pruning_target_index).await?;
    let num_output_diffs = prune_output_diffs(storage, &mut batch, start_index, pruning_target_index).await?;

    let mut num_receipts = 0;
    // Add receipts optionally.
    if config.prune_receipts() {
        let receipts = collect_receipts(storage, start_index, pruning_target_index).await?;
        num_receipts += prune_receipts(storage, &mut batch, receipts).await?;
    }

    storage
        .batch_commit(batch, true)
        .await
        // If that error actually happens we set the database to 'corrupted'!
        .map_err(|e| Error::BatchCommitError(Box::new(e)))?;

    info!(
        "Pruned {} milestones, {} output diffs, {} receipts.",
        num_milestones, num_output_diffs, num_receipts
    );
    info!(
        "Pruned {} confirmed messages including {} edges, {} indexations.",
        num_messages, num_edges, num_indexations
    );
    info!(
        "Pruned {} received ids, including {} messages, {} edges, {} indexations.",
        num_received_ids, num_unconf_messages, num_unconf_edges, num_unconf_indexations
    );

    tangle.update_pruning_index(pruning_target_index);
    info!("Pruning index now at {}.", pruning_target_index);

    bus.dispatch(PrunedIndex {
        index: pruning_target_index,
    });

    Ok(())
}

async fn prune_messages<S: StorageBackend, M: IntoIterator<Item = MessageId>>(
    storage: &S,
    batch: &mut S::Batch,
    messages: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for message_id in messages.into_iter() {
        // "&StorageHooks(ResourceHandle(B))": *** => B
        Batch::<MessageId, Message>::batch_delete(storage, batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, &message_id)
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_indexations<S: StorageBackend, I: IntoIterator<Item = (PaddedIndex, MessageId)>>(
    storage: &S,
    batch: &mut S::Batch,
    indexes: I,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (index, message_id) in indexes.into_iter() {
        Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, &(index, message_id))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_edges<S: StorageBackend, E: IntoIterator<Item = Edge>>(
    storage: &S,
    batch: &mut S::Batch,
    edges: E,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (from, to) in edges.into_iter().map(|edge| (edge.from_parent, edge.to_child)) {
        Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, &(from, to))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_received_ids<S: StorageBackend, M: IntoIterator<Item = (MilestoneIndex, ReceivedMessageId)>>(
    storage: &S,
    batch: &mut S::Batch,
    received: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (received_at, received_message_id) in received.into_iter() {
        Batch::<(MilestoneIndex, ReceivedMessageId), ()>::batch_delete(
            storage,
            batch,
            &(received_at, received_message_id),
        )
        .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_milestones<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, Milestone>::batch_delete(storage, batch, &index.into())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

async fn prune_output_diffs<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index.into())
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }
    Ok(num_pruned)
}

async fn prune_receipts<S: StorageBackend, R: IntoIterator<Item = (MilestoneIndex, Receipt)>>(
    storage: &S,
    batch: &mut S::Batch,
    receipts: R,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (index, receipt) in receipts.into_iter() {
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index, receipt))
            .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

mod debugging {
    use super::*;

    // This checklist ensures, that no confirmed message will be collected twice by the past-cone
    // traversal of 'target_index'.
    pub fn unique_confirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
        static INSTANCE1: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
        INSTANCE1.get_or_init(|| Mutex::new(HashMap::default()))
    }

    // This checklist ensures, that no unconfirmed message is referenced under two different milestone
    // indexes.
    pub fn unique_unconfirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
        static INSTANCE2: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
        INSTANCE2.get_or_init(|| Mutex::new(HashMap::default()))
    }
}
