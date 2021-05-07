// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::{types::Receipt, workers::storage::StorageBackend};

use bee_message::prelude::{Essence, IndexationPayload, Message, MessageId, MilestoneIndex, PaddedIndex, Payload};
use bee_storage::access::Fetch;
use bee_tangle::{solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage, MsTangle};

use hashbrown::{HashMap, HashSet};
use log::error;
use ref_cast::RefCast;

use std::collections::VecDeque;

pub type Messages = HashSet<MessageId>;
pub type Edges = HashSet<Edge>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub type Indexations = Vec<(PaddedIndex, MessageId)>;
pub type UnreferencedMessages = Vec<(MilestoneIndex, UnreferencedMessage)>;
pub type Receipts = Vec<(MilestoneIndex, Receipt)>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

/// Collects all prunable nodes/vertices and edges of the Tangle for the specified index.
pub async fn collect_confirmed_data<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    target_index: MilestoneIndex,
    old_seps: &Seps,
) -> Result<(Messages, Edges, Seps, Indexations), Error> {
    let mut collected_messages = Messages::default();
    let mut edges = Edges::default();
    let mut new_seps = Seps::default();
    let mut indexations = Indexations::default();

    // Query the Tangle for the `message_id` of `root_index`, that is the root of the past-cone that we're about to
    // traverse.
    let target_id = tangle
        .get_milestone_message_id(target_index)
        .await
        // `unwrap` should be safe since we can assume at this point the underlying db is not corrupted.
        // alternative:
        // .ok_or(Error::MilestoneNotFoundInTangle(*current_index))?;
        .unwrap_or_else(|| {
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
    let mut num_redundant: usize = 0;

    // Each new milestone cone fully contains the previous cone. That means, that

    while let Some(current_id) = parents.pop_front() {
        // Skip conditions that will make the traversal terminate eventually:
        // (1) already processed that message (since children share parents)
        // (2) hit an old SEP (reached the "bottom" of the cone)
        if collected_messages.contains(&current_id) || old_seps.contains_key(SolidEntryPoint::ref_cast(&current_id)) {
            continue;
        } else {
            // We must be able to get its parents (unless the db is corrupt)
            if let Some((maybe_payload, current_parents)) = Fetch::<MessageId, Message>::fetch(storage, &current_id)
                .await
                .map_err(|e| Error::StorageError(Box::new(e)))?
                // TODO: removing Copying/Cloning
                .map(|msg| (msg.payload().clone(), msg.parents().iter().copied().collect::<Vec<_>>()))
            {
                // Collect possible indexation payloads.
                if let Some(indexation) = unwrap_indexation(maybe_payload) {
                    let padded_index = indexation.padded_index();

                    indexations.push((padded_index, current_id));
                }

                // Collect edges.
                for parent_id in &current_parents {
                    let _ = edges.insert(Edge {
                        from_parent: *parent_id,
                        to_child: current_id,
                    });
                }

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

            let _ = collected_messages.insert(current_id);

            // We only add this as a new SEP if it has at least one unconfirmed aprover.
            //
            // TODO: Replace with "direct storage access" to not pull them all into the cache again.
            //
            // `unwrap` should be safe, because the current message has been confirmed, and hence must have
            // approvers.
            let approvers = tangle.get_children(&current_id).await.unwrap();

            // If all approvers are part of the `collected_messages` set already, then we can assume that this
            // message is redundant for our new SEP set, because there is no path to it without visiting its also
            // confirmed children. Since we traverse the past-cone breadth-first we can be sure that its
            // children are visited before, and the following skip condition is triggered often. This allows
            // us to not having to fetch the metadata for all of its approvers from the Tangle.
            //
            // This is an efficient method to find the "surface" of a confirmed past-cone, that is all confirmed
            // messages, that have at least one child not confirmed by this milestone (target_index + x).
            //
            if approvers
                .iter()
                .all(|approver_id| collected_messages.contains(approver_id))
            {
                num_redundant += 1;
                continue;
            } else {
                // We know now that some message of a future cone must reference it, and hence, we need to keep it.
                let _ = new_seps.insert(current_id.into(), target_index);
            }
        }
    }

    dbg!(num_redundant, new_seps.len(), collected_messages.len());

    Ok((collected_messages, edges, new_seps, indexations))
}

pub async fn collect_still_unconfirmed_data<S: StorageBackend>(
    storage: &S,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(UnreferencedMessages, Messages, Edges, Indexations), Error> {
    let mut received = UnreferencedMessages::default();
    let mut messages = Messages::default();
    let mut edges = Edges::default();
    let mut indexations = Indexations::default();

    for index in *start_index..=*target_index {
        collect_unconfirmed_data_by_index(
            storage,
            index.into(),
            &mut received,
            &mut messages,
            &mut edges,
            &mut indexations,
        )
        .await?;
    }

    Ok((received, messages, edges, indexations))
}

/// Get the unconfirmed/unreferenced messages.
async fn collect_unconfirmed_data_by_index<S: StorageBackend>(
    storage: &S,
    index: MilestoneIndex,
    received: &mut UnreferencedMessages,
    messages: &mut Messages,
    edges: &mut Edges,
    indexations: &mut Indexations,
) -> Result<(), Error> {
    let fetched = Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &index)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?
        // `unwrap` is safe, because the value returned is a `Vec`, hence if nothing can be fetched the result will be
        // `Some(vec![])`
        .unwrap();

    if fetched.is_empty() {
        return Ok(());
    }

    for received_message_id in fetched.iter().map(|msg| msg.message_id()) {
        // **NOTE**: It is very often the case, that the `Fetch` will not succeed, because the data has been deleted
        // already from the database via a previous pruning run. So this here is just to prune the unconfirmed, and
        // hence untraversed remnants.

        if let Some((maybe_payload, parents)) = Fetch::<MessageId, Message>::fetch(storage, &received_message_id)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?
            // TODO: remove the Clone/Copy
            .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
        {
            // Collect messages (or rather the message ids pointing to still existing messages)
            messages.insert(*received_message_id);

            // Collect possible indexation payloads
            if let Some(indexation) = unwrap_indexation(maybe_payload) {
                let padded_index = indexation.padded_index();
                let message_id = *received_message_id;

                indexations.push((padded_index, message_id));
            }

            // Collect edges
            for parent in parents.iter() {
                edges.insert(Edge {
                    from_parent: *parent,
                    to_child: *received_message_id,
                });
            }
        }
    }

    // Unconfirmed messages associated with their milestone index, which we need for the batch operation.
    received.extend(std::iter::repeat(index).zip(fetched.into_iter()));

    Ok(())
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

pub async fn collect_receipts<S: StorageBackend>(
    storage: &S,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<Receipts, Error> {
    let mut receipts = Receipts::default();

    for index in *start_index..=*target_index {
        let fetched = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index.into())
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?
            // TODO: explain why there are always unconfirmed messages per each milestone
            .unwrap();

        if fetched.is_empty() {
            continue;
        }

        receipts.extend(std::iter::repeat(index.into()).zip(fetched.into_iter()));
    }

    Ok(receipts)
}

/// Collects unconfirmed messages by walking the future cones of all confirmed messages by a particular milestone.
#[allow(dead_code)]
pub async fn collect_unconfirmed_data_improved<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    _target_index: MilestoneIndex,
) -> Result<(Messages, Edges, Seps), Error> {
    // This is an alternative way of fetching unconfirmed messages, which could help reduce database operations.
    todo!()
}
