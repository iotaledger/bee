// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

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
use log::error;
use ref_cast::RefCast;

use std::collections::VecDeque;

pub type Messages = HashSet<MessageId>;
pub type BufferedApprovers = HashMap<MessageId, bool>;
pub type OldSeps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub type NewSeps = OldSeps;

pub type Edges = HashSet<Edge>;
pub type Indexations = Vec<(PaddedIndex, MessageId)>;

pub type UnreferencedMessages = Vec<(MilestoneIndex, UnreferencedMessage)>;
pub type Receipts = Vec<(MilestoneIndex, Receipt)>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

/// TODO
///
/// Returns the new set of solid entry points.
pub async fn add_confirmed_data<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    batch: &mut S::Batch,
    target_index: MilestoneIndex,
    old_seps: &OldSeps,
) -> Result<(usize, usize, usize, NewSeps), Error> {
    let mut visited = Messages::default();
    let mut buffered_approvers = BufferedApprovers::default();
    let mut new_seps = NewSeps::default();

    let mut num_edges: usize = 0;
    let mut num_indexations: usize = 0;

    // Query the Tangle for the `message_id` of `root_index`, that is the root of the past-cone that we're about to
    // traverse.
    let target_id = tangle
        .get_milestone_message_id(target_index)
        .await
        // `unwrap` should be safe since we can assume at this point the underlying db is not corrupted.
        // alternative:
        // .ok_or(Error::MilestoneNotFoundInTangle(*current_index))?;
        .unwrap_or_else(|| {
            // TODO: return error
            error!(
                "Fetching milestone id for target index {} failed. This is a bug!",
                target_index
            );
            panic!("Fetching milestone id");
        });

    // We use a `VecDeque` here to traverse the past-cone in breadth-first manner. This gives use the necessary
    // guarantee that we "see" the children before any of their parents. We can use this guarantee to decide whether
    // within one past-cone of a milestone a message would be a redundant SEP, because all of its children/approvers are
    // already confirmed by the same or another milestone, and hence can be ignored.
    let mut parents: VecDeque<_> = vec![target_id].into_iter().collect();

    while let Some(current_id) = parents.pop_front() {
        // Skip conditions that will make the traversal terminate eventually:
        //      (1) already processed that message (children may share parents)
        //      (2) hit an old SEP (reached the "bottom" of the cone)
        if visited.contains(&current_id) || old_seps.contains_key(SolidEntryPoint::ref_cast(&current_id)) {
            continue;
        } else {
            // We must be able to get its parents (unless the db is corrupt)
            if let Some((maybe_payload, current_parents)) = Fetch::<MessageId, Message>::fetch(storage, &current_id)
                .await
                .map_err(|e| Error::PruningFailed(Box::new(e)))?
                .map(|msg| (msg.payload().clone(), msg.parents().iter().copied().collect::<Vec<_>>()))
            {
                // Batch possible indexation payloads.
                if let Some(indexation) = unwrap_indexation(maybe_payload) {
                    let padded_index = indexation.padded_index();

                    add_indexation_data(storage, batch, &(padded_index, current_id))?;
                    num_indexations += 1;
                }

                // Batch edges with parents.
                for parent_id in &current_parents {
                    add_edge(storage, batch, &(*parent_id, current_id))?;
                    num_edges += 1;
                }

                // Add the parents to the traversal list.
                parents.append(&mut current_parents.into_iter().collect());
            } else {
                // This error should never happen, because we made sure, that pruning of still "unconfirmed" messages
                // happens at a much later point in time, so it should not ever interfere with the pruning of
                // "confirmed" messages.
                error!(
                    "Fetching message data for confirmed_message {} failed. This is a bug!",
                    current_id
                );
            }

            let _ = visited.insert(current_id);

            add_message_and_metadata(storage, batch, &current_id)?;

            // Note:
            //
            // If all approvers are part of the `collected_messages` set already, then we can assume that this
            // message is redundant for our new SEP set, because there is no path to it without visiting its also
            // confirmed children. Since we traverse the past-cone breadth-first we can be sure that its
            // children are visited before, and the following skip condition is triggered often. This allows
            // us to not having to fetch the metadata for all of its approvers from the Tangle.
            //
            // This is an efficient method to find the "surface" of a confirmed past-cone, that is all confirmed
            // messages, that have at least one child not confirmed by this milestone (target_index + x).
            //
            // We only mark this message as a new SEP if its approvers:
            //  (a) contain at least one that is still unconfirmed;
            //  (b) contain at least one that is confirmed by a milestone with an index > target_index;
            //
            // In order to minimize database operations, we do the following:
            //  (1) if all its approvers have been confirmed by this milestone, we don't need to query the db at all;
            //  (2) if at least one approver was confirmed by a future (not yet pruned) milestone, we look into the
            //      buffer in case it was queried before already;
            //  (3) if the buffer didn't contain the information, we query the db, and add the result to the buffer;

            // Panic:
            // Unwrapping is safe, because the current message has been confirmed and hence, must have approvers.
            let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &current_id)
                .await
                .unwrap()
                .unwrap();

            // A list of approvers not belonging to the currently traversed cone.
            // These can be:
            //  (a) unconfirmed messages
            //  (b) messages confirmed by a younger milestone that than the current target
            let not_visited_approvers = approvers
                .iter()
                .filter(|id| !visited.contains(id))
                .collect::<HashSet<_>>();

            // If it has only previously traversed approvers, we can skip, because it's redundant.
            if not_visited_approvers.is_empty() {
                continue;
            }

            // If it has unvisited approvers, then we see if we buffered their information already.
            let not_buffered_approvers = not_visited_approvers
                .iter()
                .filter(|id| !buffered_approvers.contains_key(id))
                .collect::<HashSet<_>>();

            // Fetch not yet fetched metadata, and buffer it.
            for id in not_buffered_approvers {
                let is_confirmed = if let Some(conf_index) = Fetch::<MessageId, MessageMetadata>::fetch(storage, id)
                    .await
                    .unwrap()
                    .unwrap()
                    .milestone_index()
                {
                    assert!(
                        conf_index > target_index,
                        "conf_index={0}, target_index={1}",
                        conf_index,
                        target_index
                    );
                    true
                } else {
                    false
                };

                buffered_approvers.insert(**id, is_confirmed);
            }

            // If there is any confirmed not-visited approver (confirmed by a milestone younger that target), then the
            // message becomes part of the new SEP set.
            if not_visited_approvers
                .iter()
                .any(|id| *buffered_approvers.get(id).unwrap())
            {
                let _ = new_seps.insert(current_id.into(), target_index);
            }
        }
    }

    dbg!(visited.len(), new_seps.len());

    Ok((visited.len(), num_edges, num_indexations, new_seps))
}

/// Adds a message with its associated metadata to the delete batch.
fn add_message_and_metadata<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<(), Error> {
    Batch::<MessageId, Message>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, message_id)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    Ok(())
}

/// Adds an edge to the delete batch.
fn add_edge<S: StorageBackend>(storage: &S, batch: &mut S::Batch, edge: &(MessageId, MessageId)) -> Result<(), Error> {
    Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, edge)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    Ok(())
}

/// Adds indexation data to the delete batch.
fn add_indexation_data<S: StorageBackend>(
    storage: &S,
    batch: &mut S::Batch,
    index_message_id: &(PaddedIndex, MessageId),
) -> Result<(), Error> {
    Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, index_message_id)
        .map_err(|e| Error::StorageError(Box::new(e)))?;

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
        .map_err(|e| Error::StorageError(Box::new(e)))?
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

        if let Some((maybe_payload, parents)) = Fetch::<MessageId, Message>::fetch(storage, &unconfirmed_message_id)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?
            .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
        {
            // Collect messages (or rather the message ids pointing to still existing messages)
            // messages.insert(*unconfirmed_message_id);
            add_message_and_metadata(storage, batch, &unconfirmed_message_id)?;

            // Collect possible indexation payloads
            if let Some(indexation) = unwrap_indexation(maybe_payload) {
                let padded_index = indexation.padded_index();
                let message_id = *unconfirmed_message_id;

                // indexations.push((padded_index, message_id));
                add_indexation_data(storage, batch, &(padded_index, message_id))?;
                *num_indexations += 1;
            }

            // Collect edges
            for parent in parents.iter() {
                // edges.insert(Edge {
                //     from_parent: *parent,
                //     to_child: *unconfirmed_message_id,
                // });
                add_edge(storage, batch, &(*parent, *unconfirmed_message_id))?;
                *num_edges += 1;
            }
        }

        Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
            storage,
            batch,
            &(index, (*unconfirmed_message_id).into()),
        )
        .map_err(|e| Error::StorageError(Box::new(e)))?;

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
        .map_err(|e| Error::StorageError(Box::new(e)))?;

        num_pruned += 1;
    }

    Ok(num_pruned)
}

fn unwrap_indexation(maybe_payload: Option<Payload>) -> Option<Box<IndexationPayload>> {
    match maybe_payload {
        Some(Payload::Indexation(indexation)) => Some(indexation),
        Some(Payload::Transaction(transaction)) => {
            if let Essence::Regular(essence) = transaction.essence() {
                if let Some(Payload::Indexation(indexation)) = essence.payload() {
                    Some(indexation.clone())
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
            .map_err(|e| Error::StorageError(Box::new(e)))?;

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
            .map_err(|e| Error::StorageError(Box::new(e)))?;

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
            .map_err(|e| Error::StorageError(Box::new(e)))?
            // TODO: why no panic?
            .unwrap();

        if receipts.is_empty() {
            continue;
        }

        for receipt in receipts.into_iter() {
            Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index.into(), receipt))
                .map_err(|e| Error::StorageError(Box::new(e)))?;

            num += 1;
        }
    }

    Ok(num)
}
