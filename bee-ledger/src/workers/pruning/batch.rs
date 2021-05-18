// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{error::Error, metrics::TraversalMetrics};

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
pub type Approvers = HashMap<MessageId, bool>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

pub async fn add_confirmed_data<S: StorageBackend>(
    _tangle: &MsTangle<S>,
    storage: &S,
    batch: &mut S::Batch,
    target_index: MilestoneIndex,
    old_seps: &Seps,
) -> Result<(Seps, TraversalMetrics), Error> {
    let mut visited = Messages::with_capacity(512);
    let mut buffered_approvers = Approvers::with_capacity(512);
    let mut new_seps = Seps::with_capacity(512);
    let mut metrics = TraversalMetrics::default();

    let target_milestone = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &target_index)
        .await
        .map_err(|e| Error::FetchOperation(Box::new(e)))?
        .ok_or(Error::MissingMilestone(target_index))?;

    let target_id = target_milestone.message_id().clone();

    // let target_msg = Fetch::<MessageId, Message>::fetch(storage, target_id)
    //     .await
    //     .map_err(|e| Error::FetchOperation(Box::new(e)))?
    //     .ok_or(Error::MissingMessage(*target_id))?;

    // let mut messages: VecDeque<_> = vec![target_msg].into_iter().collect();

    let mut messages: VecDeque<_> = vec![target_id].into_iter().collect();

    while let Some(current_id) = messages.pop_front() {
        // let current_id = current_msg.id().0;

        if visited.contains(&current_id) {
            metrics.msg_already_visited += 1;
            continue;
        }

        if old_seps.contains_key(SolidEntryPoint::ref_cast(&current_id)) {
            metrics.bottomed += 1;
            continue;
        }

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
            metrics.indexations += 1;
        }

        for parent_id in current_parents.iter() {
            delete_edge(storage, batch, &(*parent_id, current_id))?;
            metrics.edges += 1;
        }

        messages.extend(current_parents.iter());

        let _ = visited.insert(current_id);

        delete_message_and_metadata(storage, batch, &current_id)?;

        let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &current_id)
            .await
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
            .ok_or(Error::MissingApprovers(current_id))?;

        metrics.fetched_approvers += 1;

        let mut not_visited_approvers = approvers.into_iter().filter(|id| !visited.contains(id)).peekable();

        // If all approvers were visited before, this confirmed message is a redundant SEP.
        if not_visited_approvers.peek().is_none() {
            metrics.all_approvers_visited += 1;
            continue;
        }

        metrics.approvers_not_visited += 1;

        // Try to use the buffer first before making any storage queries.
        if not_visited_approvers
            .clone()
            .any(|id| *buffered_approvers.get(&id).unwrap_or(&false))
        {
            new_seps.insert(current_id.into(), target_index);

            metrics.found_sep_early += 1;
            continue;
        }

        // Fetch not yet fetched metadata, and buffer it.
        for id in not_visited_approvers {
            if buffered_approvers.contains_key(&id) {
                continue;
            }

            let is_confirmed_in_future = if let Some(conf_index) =
                Fetch::<MessageId, MessageMetadata>::fetch(storage, &id)
                    .await
                    .map_err(|e| Error::FetchOperation(Box::new(e)))?
                    .ok_or(Error::MissingMetadata(id))?
                    .milestone_index()
            {
                // Note that an approver can be confirmed by the same milestone (be child and sibling at the same time).
                conf_index > target_index
            } else {
                false
            };

            buffered_approvers.insert(id, is_confirmed_in_future);

            metrics.buffered_approvers += 1;

            if is_confirmed_in_future {
                new_seps.insert(current_id.into(), target_index);

                metrics.found_sep_late += 1;
                continue;
            }
        }
    }

    metrics.new_seps = new_seps.len();

    Ok((new_seps, metrics))
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

pub async fn add_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(usize, usize, usize), Error> {
    let mut num_messages: usize = 0;
    let mut num_indexations: usize = 0;
    let mut num_edges: usize = 0;

    for index in *start_index..=*target_index {
        add_unconfirmed_data_by_index(
            storage,
            batch,
            index.into(),
            &mut num_messages,
            &mut num_edges,
            &mut num_indexations,
        )
        .await?;
    }

    Ok((num_messages, num_edges, num_indexations))
}

/// Get the unconfirmed/unreferenced messages.
async fn add_unconfirmed_data_by_index<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
    num_messages: &mut usize,
    num_edges: &mut usize,
    num_indexations: &mut usize,
) -> Result<(), Error> {
    let fetched_unconfirmed = Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
        .await
        .map_err(|e| Error::BatchOperation(Box::new(e)))?
        // Panic:
        // Unwrapping is fine, because the value returned is a `Vec`, hence if nothing can be fetched the result will be
        // `Some(vec![])`
        .unwrap();

    if fetched_unconfirmed.is_empty() {
        return Ok(());
    }

    for unconfirmed_message_id in fetched_unconfirmed.iter().map(|msg| msg.message_id()) {
        // **NOTE**:
        // It is very often the case, that the `Fetch` will not succeed, because the data has been deleted
        // already during a previous pruning. So this here is just to prune the unconfirmed, and hence untraversed
        // remnants.

        if let Some(msg) = Fetch::<MessageId, Message>::fetch(storage, unconfirmed_message_id)
            .await
            .map_err(|e| Error::FetchOperation(Box::new(e)))?
        {
            // .ok_or(Error::MissingMessage(*unconfirmed_message_id))?;

            let maybe_payload = msg.payload().as_ref();
            let parents = msg.parents();

            // if let Some((maybe_payload, parents)) = Fetch::<MessageId, Message>::fetch(storage,
            // &unconfirmed_message_id)     .await
            //     .map_err(|e| Error::BatchOperation(Box::new(e)))?
            //     .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
            // {

            // Collect messages (or rather the message ids pointing to still existing messages)
            // messages.insert(*unconfirmed_message_id);
            delete_message_and_metadata(storage, batch, &unconfirmed_message_id)?;

            // Collect possible indexation payloads
            if let Some(indexation) = unwrap_indexation(maybe_payload) {
                let padded_index = indexation.padded_index();
                let message_id = *unconfirmed_message_id;

                // indexations.push((padded_index, message_id));
                delete_indexation_data(storage, batch, &(padded_index, message_id))?;
                *num_indexations += 1;
            }

            // Collect edges
            for parent in parents.iter() {
                // edges.insert(Edge {
                //     from_parent: *parent,
                //     to_child: *unconfirmed_message_id,
                // });
                delete_edge(storage, batch, &(*parent, *unconfirmed_message_id))?;
                *num_edges += 1;
            }
        }

        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            batch,
            &(index, (*unconfirmed_message_id).into()),
        )
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

        *num_messages += 1;
    }

    Ok(())
}

async fn prune_unreferenced<S: StorageBackend, M: IntoIterator<Item = (MilestoneIndex, UnreferencedMessage)>>(
    storage: &S,
    batch: &mut S::Batch,
    received: M,
) -> Result<usize, Error> {
    let mut num_pruned = 0;

    for (received_at, received_message_id) in received.into_iter() {
        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            batch,
            &(received_at, received_message_id),
        )
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

fn unwrap_indexation(maybe_payload: Option<&Payload>) -> Option<&Box<IndexationPayload>> {
    match maybe_payload {
        Some(Payload::Indexation(indexation)) => Some(indexation),
        Some(Payload::Transaction(transaction)) => {
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

// async fn prune_messages<S: StorageBackend, M: IntoIterator<Item = MessageId>>(
//     storage: &S,
//     batch: &mut S::Batch,
//     messages: M,
// ) -> Result<usize, Error> {
//     let mut num_pruned = 0;

//     for message_id in messages.into_iter() {
//         // "&StorageHooks(ResourceHandle(B))": *** => B
//         Batch::<MessageId, Message>::batch_delete(storage, batch, &message_id)
//             .map_err(|e| Error::StorageError(Box::new(e)))?;

//         Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, &message_id)
//             .map_err(|e| Error::StorageError(Box::new(e)))?;

//         num_pruned += 1;
//     }

//     Ok(num_pruned)
// }

// async fn prune_edges<S: StorageBackend, E: IntoIterator<Item = Edge>>(
//     storage: &S,
//     batch: &mut S::Batch,
//     edges: E,
// ) -> Result<usize, Error> {
//     let mut num_pruned = 0;

//     for (from, to) in edges.into_iter().map(|edge| (edge.from_parent, edge.to_child)) {
//         Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, &(from, to))
//             .map_err(|e| Error::StorageError(Box::new(e)))?;

//         num_pruned += 1;
//     }

//     Ok(num_pruned)
// }

// async fn prune_indexations<S: StorageBackend, I: IntoIterator<Item = (PaddedIndex, MessageId)>>(
//     storage: &S,
//     batch: &mut S::Batch,
//     indexes: I,
// ) -> Result<usize, Error> {
//     let mut num_pruned = 0;

//     for (index, message_id) in indexes.into_iter() {
//         Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, &(index, message_id))
//             .map_err(|e| Error::StorageError(Box::new(e)))?;

//         num_pruned += 1;
//     }

//     Ok(num_pruned)
// }

pub async fn add_milestones<S: StorageBackend>(
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

pub async fn add_output_diffs<S: StorageBackend>(
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

pub async fn add_receipts<S: StorageBackend>(
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
