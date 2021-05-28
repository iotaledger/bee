// Copyright 2020 IOTA Stiftung
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
pub type Approvers = HashMap<MessageId, MilestoneIndex>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

pub async fn delete_confirmed_data<S: StorageBackend>(
    _tangle: &MsTangle<S>,
    storage: &S,
    batch: &mut S::Batch,
    target_index: MilestoneIndex,
    curr_seps: &Seps,
) -> Result<(Seps, ConfirmedMetrics), Error> {
    // TODO: we should probably think about not allocating those hashmaps each time.
    let mut visited = Messages::with_capacity(512);
    let mut buffered_approvers = Approvers::with_capacity(512);
    let mut new_seps = Seps::with_capacity(512);
    let mut metrics = ConfirmedMetrics::default();

    let target_milestone = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &target_index)
        .await
        .map_err(|e| Error::FetchOperation(Box::new(e)))?
        .ok_or(Error::MissingMilestone(target_index))?;

    let target_id = target_milestone.message_id().clone();

    let mut messages: VecDeque<_> = vec![target_id].into_iter().collect();

    while let Some(current_id) = messages.pop_front() {
        // Skip message if we already visited it.
        if visited.contains(&current_id) {
            metrics.msg_already_visited += 1;
            continue;
        }

        // Skip message if it's a SEP.
        if curr_seps.contains_key(SolidEntryPoint::ref_cast(&current_id)) {
            metrics.bottomed += 1;
            continue;
        }

        // Fetch the associated message to find out about its parents (and other data).
        let msg = Fetch::<MessageId, Message>::fetch(storage, &current_id)
            .await
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
            .ok_or(Error::MissingMessage(current_id))?;

        metrics.fetched_messages += 1;

        let maybe_payload = msg.payload().as_ref();
        let current_parents = msg.parents();

        if let Some(indexation) = unwrap_indexation(maybe_payload) {
            let padded_index = indexation.padded_index();

            delete_indexation_data(storage, batch, &(padded_index, current_id))?;
            metrics.prunable_indexations += 1;
        }

        for parent_id in current_parents.iter() {
            delete_edge(storage, batch, &(*parent_id, current_id))?;
            metrics.prunable_edges += 1;
        }

        // Continue the traversal with its parents.
        messages.extend(current_parents.iter());

        // Mark this message as "visited".
        visited.insert(current_id);

        delete_message_and_metadata(storage, batch, &current_id)?;

        // Fetch its approvers from storage so that we can decide whether to keep it as an SEP, or not.
        let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &current_id)
            .await
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
            .ok_or(Error::MissingApprovers(current_id))?;

        metrics.fetched_approvers += 1;

        // We need to base our decision on the approvers we haven't visited yet, more specifically the ones beyond the 'target_index'.
        let mut unvisited_approvers = approvers.into_iter().filter(|id| !visited.contains(id)).peekable();

        // If all its approvers were visited already during this traversal, this message is redundant as a SEP, and can be ignored.
        if unvisited_approvers.peek().is_none() {
            metrics.all_approvers_visited += 1;
            continue;
        }

        metrics.not_all_approvers_visited += 1;

        // To decide for how long we need to keep a particular SEP, we need to know the largest confirming index over all its approvers.
        let mut max_conf_index = target_index;
        for id in unvisited_approvers {
            let conf_index = match buffered_approvers.get(&id) {
                Some(conf_index) => *conf_index,
                None => {
                    // Fetch and buffer
                    let conf_index = if let Some(conf_index) = Fetch::<MessageId, MessageMetadata>::fetch(storage, &id)
                        .await
                        .map_err(|e| Error::FetchOperation(Box::new(e)))?
                        .ok_or(Error::MissingMetadata(id))?
                        .milestone_index()
                    {
                        // Note that an approver can be confirmed by the same milestone (be child and sibling at the same time).
                        conf_index
                    } else {
                        target_index
                    };

                    buffered_approvers.insert(id, conf_index);

                    metrics.buffered_approvers += 1;

                    conf_index
                }
            };

            if conf_index > max_conf_index {
                max_conf_index = conf_index;
            }
        }

        if max_conf_index > target_index {
            new_seps.insert(current_id.into(), max_conf_index);

            metrics.found_seps += 1;
        }
    }

    metrics.prunable_messages = visited.len();
    metrics.new_seps = new_seps.len();

    Ok((new_seps, metrics))
}

pub async fn delete_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<UnconfirmedMetrics, Error> {
    let mut metrics = UnconfirmedMetrics::default();

    for index in *start_index..=*target_index {
        let unconf_msgs = match Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index.into())
            .await
            .map_err(|e| Error::BatchOperation(Box::new(e)))?
        {
            Some(unconf_msgs) => {
                if unconf_msgs.is_empty() {
                    metrics.no_unconfirmed += 1;
                    continue;
                } else {
                    unconf_msgs
                }
            }
            None => {
                metrics.no_unconfirmed += 1;
                continue;
            }
        };

        // TODO: use MultiFetch
        for unconf_msg_id in unconf_msgs.iter().map(|unconf_msg| unconf_msg.message_id()) {
            // Skip those that were confirmed.
            match Fetch::<MessageId, MessageMetadata>::fetch(storage, unconf_msg_id)
                .await
                .map_err(|e| Error::FetchOperation(Box::new(e)))?
            {
                Some(msg_meta) => {
                    if msg_meta.milestone_index().is_some() {
                        metrics.was_confirmed += 1;
                        continue;
                    }
                }
                None => {
                    metrics.already_pruned += 1;
                    continue;
                }
            }

            // Delete those messages that remained unconfirmed.
            match Fetch::<MessageId, Message>::fetch(storage, unconf_msg_id)
                .await
                .map_err(|e| Error::FetchOperation(Box::new(e)))?
            {
                Some(msg) => {
                    let maybe_payload = msg.payload().as_ref();
                    let parents = msg.parents();

                    // Add message data to the delete batch.
                    delete_message_and_metadata(storage, batch, &unconf_msg_id)?;

                    if let Some(indexation) = unwrap_indexation(maybe_payload) {
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
                &(index.into(), (*unconf_msg_id).into()),
            )
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;

            metrics.prunable_messages += 1;
        }
    }

    Ok(metrics)
}

/// Adds a message with its associated metadata to the delete batch.
fn delete_message_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<(), Error> {
    Batch::<MessageId, Message>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

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
    Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, index_message_id)
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

    Ok(())
}

pub async fn delete_milestones<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, Milestone>::batch_delete(storage, batch, &index.into())
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}

pub async fn delete_output_diffs<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num = 0;

    for index in *start_index..=*target_index {
        Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index.into())
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;

        num += 1;
    }
    Ok(num)
}

pub async fn delete_receipts<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<usize, Error> {
    let mut num = 0;

    for index in *start_index..=*target_index {
        let receipts = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index.into())
            .await
            .map_err(|e| Error::BatchOperation(Box::new(e)))?
            // TODO: why no panic?
            .unwrap();

        if receipts.is_empty() {
            continue;
        }

        for receipt in receipts.into_iter() {
            Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index.into(), receipt))
                .map_err(|e| Error::BatchOperation(Box::new(e)))?;

            num += 1;
        }
    }

    Ok(num)
}

fn unwrap_indexation(maybe_payload: Option<&Payload>) -> Option<&Box<IndexationPayload>> {
    match maybe_payload {
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
