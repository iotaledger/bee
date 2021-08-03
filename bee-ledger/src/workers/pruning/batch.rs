// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    error::Error,
    metrics::{ConfirmedMetrics, UnconfirmedMetrics},
};

use crate::{
    types::{OutputDiff, Receipt},
    workers::storage::StorageBackend,
};

use bee_message::{
    milestone::Milestone,
    prelude::{Essence, IndexationPayload, Message, MessageId, MilestoneIndex, PaddedIndex, Payload},
};
use bee_storage::access::{Batch, Fetch};
use bee_tangle::{
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage, MsTangle,
};

use hashbrown::{HashMap, HashSet};
use ref_cast::RefCast;

use std::collections::VecDeque;

pub type Messages = HashSet<MessageId>;
pub type ApproverCache = HashMap<MessageId, MilestoneIndex>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

pub async fn delete_confirmed_data<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    current_seps: &Seps,
) -> Result<(Seps, ConfirmedMetrics), Error> {
    // A list of already visited messages.
    let mut visited = Messages::with_capacity(512);
    // A cache to prevent unnecessary double fetches from the storage.
    let mut approver_cache = ApproverCache::with_capacity(512);
    // The new SEPs for this cone.
    let mut new_seps = Seps::with_capacity(512);
    let mut metrics = ConfirmedMetrics::default();

    // Get the `MessageId` of the milestone that should be pruned from the storage.
    let prune_ms = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &prune_index)
        .map_err(|e| Error::FetchOperation(Box::new(e)))?
        .ok_or(Error::MissingMilestone(prune_index))?;

    let prune_ms_id = *prune_ms.message_id();

    let mut to_visit: VecDeque<_> = vec![prune_ms_id].into_iter().collect();

    while let Some(message_id) = to_visit.pop_front() {
        // Skip already visited messages.
        if visited.contains(&message_id) {
            metrics.msg_already_visited += 1;
            continue;
        }

        // Skip SEPs.
        if current_seps.contains_key(SolidEntryPoint::ref_cast(&message_id)) {
            metrics.references_sep += 1;
            continue;
        }

        // Get the `Message` for `message_id`.
        let msg = match Fetch::<MessageId, Message>::fetch(storage, &message_id)
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
            .ok_or(Error::MissingMessage(message_id))
        {
            Ok(msg) => msg,
            Err(e) => {
                log::error!(
                    "failed to fetch `Message` associated with message id {} during past-cone traversal of milestone {} ({})",
                    &message_id,
                    &prune_index,
                    &prune_ms_id,
                );

                return Err(e);
            }
        };

        // Delete `Indexation` payloads (if existent).
        let payload = msg.payload().as_ref();
        if let Some(indexation) = unwrap_indexation(payload) {
            let padded_index = indexation.padded_index();

            delete_indexation_data(storage, batch, &(padded_index, message_id))?;
            metrics.prunable_indexations += 1;
        }

        // Delete edges.
        let parents = msg.parents();
        for parent_id in parents.iter() {
            delete_edge(storage, batch, &(*parent_id, message_id))?;
            metrics.prunable_edges += 1;
        }

        // Add its parents to the list of yet to traverse messages.
        to_visit.extend(parents.iter().copied());

        // Mark this message as "visited".
        visited.insert(message_id);

        delete_message_and_metadata(storage, batch, &message_id)?;

        // --- Everything that follows is required to decide whether the message id of pruned message should be kept as
        // a solid entry point. We keep the set of SEPs as minimal as possible by checking whether there are
        // still messages in future cones (beyond the new pruning index) that are referencing the current
        // message (similar to a garbage collector). ---

        // Fetch its approvers from storage so that we can decide whether to keep it as an SEP, or not.
        let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &message_id)
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
            .ok_or(Error::MissingApprovers(message_id))?;

        let mut unvisited_approvers = approvers.into_iter().filter(|id| !visited.contains(id)).peekable();

        if unvisited_approvers.peek().is_none() {
            metrics.all_approvers_visited += 1;

            // Ignore message whose children are all part of this cone.
            continue;
        }

        metrics.not_all_approvers_visited += 1;

        // To decide for how long we need to keep a particular SEP around, we need to know the largest confirming index
        // of all its approvers. We initialize this value with the lower bound.
        let mut max_conf_index = *prune_index;

        for approver_id in unvisited_approvers {
            let approver_conf_index = match approver_cache.get(&approver_id) {
                Some(conf_index) => {
                    // We fetched the metadata of this approver before (fast path).
                    metrics.approver_cache_hit += 1;

                    **conf_index
                }
                None => {
                    // We need to fetch the metadata of this approver (slow path).
                    metrics.approver_cache_miss += 1;

                    let approver_md = Fetch::<MessageId, MessageMetadata>::fetch(storage, &approver_id)
                        .map_err(|e| Error::FetchOperation(Box::new(e)))?
                        .ok_or(Error::MissingMetadata(approver_id))?;

                    // FIXME: temporary consistency check
                    let cached_approver_md = tangle
                        .get_metadata(&approver_id)
                        .await
                        .ok_or(Error::MissingMetadata(approver_id))?;

                    if cached_approver_md.milestone_index().is_some() != approver_md.milestone_index().is_some()
                        || cached_approver_md.flags().is_referenced() != approver_md.flags().is_referenced()
                    {
                        log::error!("Cache and Storage have inconsistent metadata for {}", approver_id);
                    }

                    // Note that an approver can still be confirmed by the same milestone despite the breadth-first walk
                    // (if it is child/approver and sibling at the same time), in other words:
                    // conf_index = prune_index is possible.
                    let conf_index = approver_md.milestone_index().unwrap_or_else(|| {
                        // ---
                        // BUG/FIXME: The invariant ".flags().is_referenced() => (milestone_index().is_some() == true)
                        // is violated" due to some bug in our tangle impl (probably
                        // `update_metadata`), hence we need this mitigation code for now. ---
                        log::trace!(
                            "Bug mitigation: Set {}+20 to un/confirmed approver {}",
                            prune_index,
                            &approver_id
                        );

                        prune_index + 20 // BMD + 5
                    });

                    // Update the approver cache.
                    approver_cache.insert(approver_id, conf_index);

                    *conf_index
                }
            };

            max_conf_index = max_conf_index.max(approver_conf_index);
        }

        // If the greatest confirmation index of all its approvers is greater than the index we're pruning, then we need
        // to keep its message id as a solid entry point.
        if max_conf_index > *prune_index {
            new_seps.insert(message_id.into(), max_conf_index.into());

            log::trace!("New SEP: {} until {}", message_id, max_conf_index);

            metrics.found_seps += 1;
        }
    }

    metrics.prunable_messages = visited.len();
    metrics.new_seps = new_seps.len();

    if let Some(youngest_approver) = new_seps.get(&SolidEntryPoint::ref_cast(&prune_ms_id)) {
        if youngest_approver <= &prune_index {
            log::error!("Target milestone must have younger approver.");
            panic!();
        }
    } else {
        log::error!("Target milestone not included in new SEP set. This is a bug!");
        panic!();
    }

    Ok((new_seps, metrics))
}

pub async fn delete_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
) -> Result<UnconfirmedMetrics, Error> {
    let mut metrics = UnconfirmedMetrics::default();

    let unconf_msgs = match Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &prune_index)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?
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

    // TODO: use MultiFetch
    'outer_loop: for unconf_msg_id in unconf_msgs.iter().map(|unconf_msg| unconf_msg.message_id()) {
        // Skip those that were confirmed.
        match Fetch::<MessageId, MessageMetadata>::fetch(storage, unconf_msg_id)
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
        {
            Some(msg_meta) => {
                if msg_meta.flags().is_referenced() {
                    // if msg_meta.milestone_index().is_some() {
                    metrics.were_confirmed += 1;
                    continue;
                } else {
                    log::trace!("referenced flag not set for {}", unconf_msg_id);

                    // ---
                    // BUG/FIXME: The invariant ".flags().is_referenced() => (milestone_index().is_some() == true) is
                    // violated" due to some bug in our tangle impl (probably `update_metadata`),
                    // hence we need this mitigation code for now.
                    //
                    // Bug mitigation: We only prune the messasge if all its approvers are also marked as "not
                    // referenced".
                    //---
                    let unconf_approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &unconf_msg_id)
                        .map_err(|e| Error::FetchOperation(Box::new(e)))?
                        .ok_or(Error::MissingApprovers(*unconf_msg_id))?;

                    // If there's only one approver that was confirmed, then we don't prune it despite the flag
                    // indicating otherwise.
                    for approver_id in unconf_approvers {
                        if let Some(approver_md) = Fetch::<MessageId, MessageMetadata>::fetch(storage, &approver_id)
                            .map_err(|e| Error::FetchOperation(Box::new(e)))?
                        {
                            if approver_md.flags().is_referenced() {
                                continue 'outer_loop;
                            }
                        }
                    }

                    log::trace!("All approvers of {} don't have the \"referenced\" flag", unconf_msg_id);
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue;
            }
        }

        // Delete those messages that remained unconfirmed.
        match Fetch::<MessageId, Message>::fetch(storage, unconf_msg_id)
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
        {
            Some(msg) => {
                let payload = msg.payload().as_ref();
                let parents = msg.parents();

                // Add message data to the delete batch.
                delete_message_and_metadata(storage, batch, &unconf_msg_id)?;

                log::trace!("Pruned unconfirmed msg {} at {}.", unconf_msg_id, prune_index);

                if let Some(indexation) = unwrap_indexation(payload) {
                    let padded_index = indexation.padded_index();
                    let message_id = *unconf_msg_id;

                    // Add prunable indexations to the delete batch.
                    delete_indexation_data(storage, batch, &(padded_index, message_id))?;

                    metrics.prunable_indexations += 1;
                }

                // Add prunable edges to the delete batch.
                for parent in parents.iter() {
                    delete_edge(storage, batch, &(*parent, *unconf_msg_id))?;

                    metrics.prunable_edges += 1;
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue;
            }
        }

        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            batch,
            &(prune_index, (*unconf_msg_id).into()),
        )
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

        metrics.prunable_messages += 1;
    }

    Ok(metrics)
}

/// Adds a message with its associated metadata to the delete batch.
fn delete_message_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<(), Error> {
    // Message
    Batch::<MessageId, Message>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    // MessageMetadata
    Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

/// Adds an edge to the delete batch.
fn delete_edge<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    edge: &(MessageId, MessageId),
) -> Result<(), Error> {
    // Edge
    Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, edge)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

/// Adds indexation data to the delete batch.
fn delete_indexation_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index_message_id: &(PaddedIndex, MessageId),
) -> Result<(), Error> {
    // Indexation
    Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, index_message_id)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

pub async fn delete_milestone<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    // Milestone
    Batch::<MilestoneIndex, Milestone>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

pub async fn delete_output_diff<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    // OutputDiff
    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

pub async fn delete_receipts<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<usize, Error> {
    let receipts = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?
        // TODO: why no panic?
        .unwrap();

    let mut num = 0;
    for receipt in receipts.into_iter() {
        // Receipt
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index, receipt))
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;

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
