// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use bee_message::{
    milestone::{Milestone, MilestoneIndex},
    output::OutputId,
    payload::{
        indexation::{IndexationPayload, PaddedIndex},
        transaction::Essence,
        Payload,
    },
    Message, MessageId,
};
use bee_storage::backend::StorageBackendExt;
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};
use hashbrown::{HashMap, HashSet};
use ref_cast::RefCast;

use crate::{
    types::{ConsumedOutput, CreatedOutput, OutputDiff, Receipt},
    workers::{
        pruning::{
            error::Error,
            metrics::{ConfirmedDataPruningMetrics, MilestoneDataPruningMetrics, UnconfirmedDataPruningMetrics},
        },
        storage::StorageBackend,
    },
};

pub type Messages = HashSet<MessageId>;
pub type ApproverCache = HashMap<MessageId, MilestoneIndex>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

pub fn batch_prunable_confirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    current_seps: &Seps,
) -> Result<(Seps, ConfirmedDataPruningMetrics), Error> {
    // We keep a list of already visited messages.
    let mut visited = Messages::with_capacity(512);
    // We keep a cache of approvers to prevent fetch the same data from the storage more than once.
    let mut approver_cache = ApproverCache::with_capacity(512);
    // We collect new SEPs during the traversal, and return them as a result of this function.
    let mut new_seps = Seps::with_capacity(512);
    // We collect stats during the traversal, and return them as a result of this function.
    let mut metrics = ConfirmedDataPruningMetrics::default();

    // Get the `MessageId` of the milestone we are about to prune from the storage.
    let prune_id = *storage
        .fetch::<MilestoneIndex, Milestone>(&prune_index)
        .map_err(|e| Error::Storage(Box::new(e)))?
        .ok_or(Error::MissingMilestone(prune_index))?
        .message_id();

    // Breadth-first traversal will increase our chances of sorting out redundant messages without querying the storage.
    let mut to_visit: VecDeque<_> = vec![prune_id].into_iter().collect();

    while let Some(message_id) = to_visit.pop_front() {
        // Skip already visited messages.
        if visited.contains(&message_id) {
            metrics.msg_already_visited += 1;
            continue;
        }

        // Skip SEPs (from the previous pruning run).
        if current_seps.contains_key(SolidEntryPoint::ref_cast(&message_id)) {
            metrics.references_sep += 1;
            continue;
        }

        // Get the `Message` for `message_id`.
        let msg = match storage
            .fetch::<MessageId, Message>(&message_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
            .ok_or(Error::MissingMessage(message_id))
        {
            Ok(msg) => msg,
            Err(e) => {
                // Note: if we land here, then one of those things can have happened:
                // (a) the storage has been messed with, and is therefore faulty,
                // (b) the algo didn't turn a confirmed message into an SEP although it should have (bug),
                // (c) the algo treated a in fact confirmed message as unconfirmed, and removed it (bug).
                log::error!(
                    "failed to fetch `Message` associated with message id {} during past-cone traversal of milestone {} ({})",
                    &message_id,
                    &prune_index,
                    &prune_id,
                );

                return Err(e);
            }
        };

        // Delete its `Indexation` payload (if existent).
        let payload = msg.payload().as_ref();
        if let Some(indexation) = unwrap_indexation(payload) {
            let padded_index = indexation.padded_index();

            prune_indexation_data(storage, batch, &(padded_index, message_id))?;
            metrics.prunable_indexations += 1;
        }

        // Delete its edges.
        let parents = msg.parents();
        for parent_id in parents.iter() {
            prune_edge(storage, batch, &(*parent_id, message_id))?;
            metrics.prunable_edges += 1;
        }

        // Add its parents to the queue of yet to traverse messages.
        to_visit.extend(msg.into_parents().iter());

        // Remember that we've seen this message already.
        visited.insert(message_id);

        // Delete its associated data.
        prune_message_and_metadata(storage, batch, &message_id)?;

        // ---
        // Everything that follows is required to decide whether this message's id should be kept as a solid entry
        // point. We keep the set of SEPs minimal by checking whether there are still messages in future
        // milestone cones (beyond the current target index) that are referencing the currently processed
        // message (similar to a garbage collector we remove objects only if nothing is referencing it anymore).
        // ---

        // Fetch its approvers from the storage.
        let approvers = storage
            .fetch::<MessageId, Vec<MessageId>>(&message_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
            .ok_or(Error::MissingApprovers(message_id))?;

        // We can safely skip messages whose approvers are all part of the currently pruned cone. If we are lucky
        // (chances are better with the chosen breadth-first traversal) we've already seen all of its approvers.
        let mut unvisited_approvers = approvers.into_iter().filter(|id| !visited.contains(id)).peekable();
        if unvisited_approvers.peek().is_none() {
            metrics.all_approvers_visited += 1;
            continue;
        }

        metrics.not_all_approvers_visited += 1;

        // To decide for how long we need to keep a particular SEP around, we need to know the greatest confirming index
        // taken over all its approvers. We initialise this value with the lowest possible value (the current pruning
        // target index).
        let mut max_conf_index = *prune_index;

        for unvisited_id in unvisited_approvers {
            let approver_conf_index = if let Some(conf_index) = approver_cache.get(&unvisited_id) {
                // We fetched the metadata of this approver before (fast path).
                metrics.approver_cache_hit += 1;

                **conf_index
            } else {
                // We need to fetch the metadata of this approver (slow path).
                metrics.approver_cache_miss += 1;

                let unvisited_md = storage
                    .fetch::<MessageId, MessageMetadata>(&unvisited_id)
                    .map_err(|e| Error::Storage(Box::new(e)))?
                    .ok_or(Error::MissingMetadata(unvisited_id))?;

                // A non-existing milestone index means that a message remained unconfirmed and therefore is neglibable
                // for its parent in terms of SEP consideration. This can be expressed by assigning the
                // current pruning index.
                let conf_index = unvisited_md.milestone_index().unwrap_or_else(|| {
                    log::trace!("Unconfirmed approver (no milestone index): {unvisited_id}");
                    prune_index
                });

                // Update the approver cache.
                approver_cache.insert(unvisited_id, conf_index);

                *conf_index
            };

            max_conf_index = max_conf_index.max(approver_conf_index);
        }

        // If the highest confirmation index of all its approvers is greater than the index we're pruning, then we need
        // to keep its message id as a solid entry point.
        if max_conf_index > *prune_index {
            new_seps.insert(message_id.into(), max_conf_index.into());

            log::trace!("New SEP: {} until {}", message_id, max_conf_index);

            metrics.found_seps += 1;
        }
    }

    metrics.prunable_messages = visited.len();
    metrics.new_seps = new_seps.len();

    Ok((new_seps, metrics))
}

pub fn batch_prunable_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
) -> Result<UnconfirmedDataPruningMetrics, Error> {
    let mut metrics = UnconfirmedDataPruningMetrics::default();

    let unconf_msgs = match storage
        .fetch::<MilestoneIndex, Vec<UnreferencedMessage>>(&prune_index)
        .map_err(|e| Error::Storage(Box::new(e)))?
    {
        Some(unconf_msgs) => {
            if unconf_msgs.is_empty() {
                metrics.none_received = true;
                Vec::new()
            } else {
                unconf_msgs
            }
        }
        None => {
            metrics.none_received = true;
            Vec::new()
        }
    };

    // TODO: consider using `MultiFetch`
    'next_unconf_msg: for unconf_msg_id in unconf_msgs.iter().map(|unconf_msg| unconf_msg.message_id()) {
        match storage
            .fetch::<MessageId, MessageMetadata>(unconf_msg_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
        {
            Some(msg_meta) => {
                if msg_meta.flags().is_referenced() {
                    metrics.were_confirmed += 1;
                    // Skip confirmed messages.
                    continue 'next_unconf_msg;
                }
            }
            None => {
                metrics.already_pruned += 1;
                // Skip already pruned messages.
                continue 'next_unconf_msg;
            }
        }

        // Delete those messages that remained unconfirmed.
        match storage
            .fetch::<MessageId, Message>(unconf_msg_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
        {
            Some(msg) => {
                let payload = msg.payload().as_ref();
                let parents = msg.parents();

                // Add message data to the delete batch.
                prune_message_and_metadata(storage, batch, unconf_msg_id)?;

                log::trace!("Pruned unconfirmed msg {} at {}.", unconf_msg_id, prune_index);

                if let Some(indexation) = unwrap_indexation(payload) {
                    let padded_index = indexation.padded_index();
                    let message_id = *unconf_msg_id;

                    // Add prunable indexations to the delete batch.
                    prune_indexation_data(storage, batch, &(padded_index, message_id))?;

                    metrics.prunable_indexations += 1;
                }

                // Add prunable edges to the delete batch.
                for parent in parents.iter() {
                    prune_edge(storage, batch, &(*parent, *unconf_msg_id))?;

                    metrics.prunable_edges += 1;
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue 'next_unconf_msg;
            }
        }

        storage
            .batch_delete::<(MilestoneIndex, UnreferencedMessage), ()>(batch, &(prune_index, (*unconf_msg_id).into()))
            .map_err(|e| Error::Storage(Box::new(e)))?;

        metrics.prunable_messages += 1;
    }

    Ok(metrics)
}

pub fn prune_milestone_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    should_prune_receipts: bool,
) -> Result<MilestoneDataPruningMetrics, Error> {
    let mut metrics = MilestoneDataPruningMetrics::default();

    prune_milestone(storage, batch, prune_index)?;

    prune_output_diff(storage, batch, prune_index)?;

    if should_prune_receipts {
        metrics.receipts = prune_receipts(storage, batch, prune_index)?;
    }

    Ok(metrics)
}

fn prune_message_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<(), Error> {
    storage
        .batch_delete::<MessageId, Message>(batch, message_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;
    storage
        .batch_delete::<MessageId, MessageMetadata>(batch, message_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_edge<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    edge: &(MessageId, MessageId),
) -> Result<(), Error> {
    storage
        .batch_delete::<(MessageId, MessageId), ()>(batch, edge)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_indexation_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index_message_id: &(PaddedIndex, MessageId),
) -> Result<(), Error> {
    storage
        .batch_delete::<(PaddedIndex, MessageId), ()>(batch, index_message_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_milestone<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<(), Error> {
    storage
        .batch_delete::<MilestoneIndex, Milestone>(batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_output_diff<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<(), Error> {
    if let Some(output_diff) = storage
        .fetch::<MilestoneIndex, OutputDiff>(&index)
        .map_err(|e| Error::Storage(Box::new(e)))?
    {
        for consumed_output in output_diff.consumed_outputs() {
            storage
                .batch_delete::<OutputId, ConsumedOutput>(batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
            storage
                .batch_delete::<OutputId, CreatedOutput>(batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }

        if let Some(_treasury_diff) = output_diff.treasury_diff() {
            // TODO
        }
    }

    storage
        .batch_delete::<MilestoneIndex, OutputDiff>(batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

fn prune_receipts<S: StorageBackend>(storage: &S, batch: &mut S::Batch, index: MilestoneIndex) -> Result<usize, Error> {
    let receipts = storage
        .fetch::<MilestoneIndex, Vec<Receipt>>(&index)
        .map_err(|e| Error::Storage(Box::new(e)))?
        // Fine since Fetch of a Vec<_> always returns Some(Vec<_>).
        .unwrap();

    let mut num = 0;
    for receipt in receipts.into_iter() {
        storage
            .batch_delete::<(MilestoneIndex, Receipt), ()>(batch, &(index, receipt))
            .map_err(|e| Error::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}

fn unwrap_indexation(payload: Option<&Payload>) -> Option<&IndexationPayload> {
    match payload {
        Some(Payload::Indexation(indexation)) => Some(indexation),
        Some(Payload::Transaction(transaction)) =>
        {
            #[allow(irrefutable_let_patterns)]
            if let Essence::Regular(essence) = transaction.essence() {
                if let Some(Payload::Indexation(indexation)) = essence.payload() {
                    Some(indexation)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

// TODO: consider using this instead of 'truncate'
#[allow(dead_code)]
fn prune_seps<S: StorageBackend>(storage: &S, batch: &mut S::Batch, seps: &[SolidEntryPoint]) -> Result<usize, Error> {
    let mut num = 0;
    for sep in seps {
        storage
            .batch_delete::<SolidEntryPoint, MilestoneIndex>(batch, sep)
            .map_err(|e| Error::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}
