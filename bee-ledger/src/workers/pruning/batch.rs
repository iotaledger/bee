// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{ConsumedOutput, CreatedOutput, OutputDiff, Receipt},
    workers::{
        consensus::worker::EXTRA_PRUNING_DEPTH,
        pruning::{
            error::Error,
            metrics::{ConfirmedDataPruningMetrics, MilestoneDataPruningMetrics, UnconfirmedDataPruningMetrics},
        },
        storage::StorageBackend,
    },
};

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

pub async fn prune_confirmed_data<S: StorageBackend>(
    tangle: &MsTangle<S>,
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

    // FIXME: mitigation code
    let mitigation_threshold = tangle.config().below_max_depth() + EXTRA_PRUNING_DEPTH; // = BMD + 5

    // Get the `MessageId` of the milestone we are about to prune from the storage.
    let prune_id = *Fetch::<MilestoneIndex, Milestone>::fetch(storage, &prune_index)
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
        let msg = match Fetch::<MessageId, Message>::fetch(storage, &message_id)
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
        let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &message_id)
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

                let unvisited_md = Fetch::<MessageId, MessageMetadata>::fetch(storage, &unvisited_id)
                    .map_err(|e| Error::Storage(Box::new(e)))?
                    .ok_or(Error::MissingMetadata(unvisited_id))?;

                // Note, that an unvisited approver of this message can still be confirmed by the same milestone
                // (despite the breadth-first traversal), if it is also its sibling.
                let conf_index = unvisited_md.milestone_index().unwrap_or_else(|| {
                    // ---
                    // BUG/FIXME:
                    // In very rare situations the milestone index has not been set for a confirmed message. If that
                    // message happens to be the one with the highest confirmation index, then the SEP created from the
                    // current message would be removed too early, i.e. before all of its referrers, and pruning would
                    // fail without a way to ever recover. We suspect the bug to be a race condition in the
                    // `update_metadata` method of the `MsTangle` implementation.
                    //
                    // Mitigation strategy:
                    // We rely on the coordinator to not confirm something that attaches to a message that was confirmed
                    // more than 20 milestones (BMD + EXTRA_PRUNING_DEPTH) ago, i.e. a lazy tip.
                    // ---
                    log::trace!(
                        "Bug mitigation: Using '{} + mitigation_threshold ({})' for approver '{}'",
                        prune_index,
                        mitigation_threshold,
                        &unvisited_id
                    );

                    prune_index + mitigation_threshold
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

pub async fn prune_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
) -> Result<UnconfirmedDataPruningMetrics, Error> {
    let mut metrics = UnconfirmedDataPruningMetrics::default();

    let unconf_msgs = match Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &prune_index)
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
    'outer_loop: for unconf_msg_id in unconf_msgs.iter().map(|unconf_msg| unconf_msg.message_id()) {
        // Skip those that were confirmed.
        match Fetch::<MessageId, MessageMetadata>::fetch(storage, unconf_msg_id)
            .map_err(|e| Error::Storage(Box::new(e)))?
        {
            Some(msg_meta) => {
                if msg_meta.flags().is_referenced() {
                    metrics.were_confirmed += 1;
                    continue;
                } else {
                    // We log which messages were never confirmed.
                    log::trace!("'referenced' flag not set for {}", unconf_msg_id);

                    // ---
                    // BUG/FIXME:
                    // In very rare situations the `referenced` flag has not been set for a confirmed message. This
                    // would lead to it being removed as an unconfirmed message causing the past-cone traversal of a
                    // milestone to fail. That would cause pruning to fail without a way to ever recover. We suspect the
                    // bug to be a race condition in the `update_metadata` method of the `MsTangle` implementation.
                    //
                    // Mitigation strategy:
                    // To make occurring this scenario sufficiently unlikely, we only prune a message with
                    // the flag indicating "not referenced", if all its approvers are also flagged as "not referenced".
                    // In other words: If we find at least one confirmed approver, then we know the flag wasn't set
                    // appropriatedly for the current message due to THE bug, and that we cannot prune it.
                    // ---
                    let unconf_approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, unconf_msg_id)
                        .map_err(|e| Error::Storage(Box::new(e)))?
                        .ok_or(Error::MissingApprovers(*unconf_msg_id))?;

                    for unconf_approver_id in unconf_approvers {
                        if let Some(unconf_approver_md) =
                            Fetch::<MessageId, MessageMetadata>::fetch(storage, &unconf_approver_id)
                                .map_err(|e| Error::Storage(Box::new(e)))?
                        {
                            if unconf_approver_md.flags().is_referenced() {
                                continue 'outer_loop;
                            }
                        }
                    }

                    log::trace!("all of '{}'s approvers are flagged 'unreferenced'", unconf_msg_id);
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue;
            }
        }

        // Delete those messages that remained unconfirmed.
        match Fetch::<MessageId, Message>::fetch(storage, unconf_msg_id).map_err(|e| Error::Storage(Box::new(e)))? {
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
                continue;
            }
        }

        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            batch,
            &(prune_index, (*unconf_msg_id).into()),
        )
        .map_err(|e| Error::Storage(Box::new(e)))?;

        metrics.prunable_messages += 1;
    }

    Ok(metrics)
}

pub async fn prune_milestone_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    should_prune_receipts: bool,
) -> Result<MilestoneDataPruningMetrics, Error> {
    let mut metrics = MilestoneDataPruningMetrics::default();

    // Add prunable milestones to the delete batch.
    prune_milestone(storage, batch, prune_index).await?;

    // Add prunable output diffs to the delete batch.
    prune_output_diff(storage, batch, prune_index).await?;

    // Add prunable receipts the delete batch, if needed.
    if should_prune_receipts {
        metrics.receipts = prune_receipts(storage, batch, prune_index).await?;
    }

    Ok(metrics)
}

/// Adds a message with its associated metadata to the delete batch.
fn prune_message_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<(), Error> {
    Batch::<MessageId, Message>::batch_delete(storage, batch, message_id).map_err(|e| Error::Storage(Box::new(e)))?;
    Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

/// Adds an edge to the delete batch.
fn prune_edge<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    edge: &(MessageId, MessageId),
) -> Result<(), Error> {
    Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, edge).map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

/// Adds indexation data to the delete batch.
fn prune_indexation_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index_message_id: &(PaddedIndex, MessageId),
) -> Result<(), Error> {
    Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, index_message_id)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

pub async fn prune_milestone<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    Batch::<MilestoneIndex, Milestone>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

pub async fn prune_output_diff<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<(), Error> {
    if let Some(output_diff) =
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index).map_err(|e| Error::Storage(Box::new(e)))?
    {
        for consumed_output in output_diff.consumed_outputs() {
            Batch::<OutputId, ConsumedOutput>::batch_delete(storage, batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
            Batch::<OutputId, CreatedOutput>::batch_delete(storage, batch, consumed_output)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?;

    Ok(())
}

pub async fn prune_receipts<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<usize, Error> {
    let receipts = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
        .map_err(|e| Error::Storage(Box::new(e)))?
        // TODO: why no panic?
        .unwrap();

    let mut num = 0;
    for receipt in receipts.into_iter() {
        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index, receipt))
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
pub async fn prune_seps<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    seps: &[SolidEntryPoint],
) -> Result<usize, Error> {
    let mut num = 0;
    for sep in seps {
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete(storage, batch, sep)
            .map_err(|e| Error::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}
