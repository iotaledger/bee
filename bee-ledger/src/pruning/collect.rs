// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::{consensus::StorageBackend, types::Receipt};

use bee_message::prelude::{Essence, HashedIndex, IndexationPayload, Message, MessageId, MilestoneIndex, Payload};
use bee_storage::access::Fetch;
use bee_tangle::{
    ms_tangle::StorageHooks, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage, MsTangle,
};

use hashbrown::{HashMap, HashSet};
use log::error;
use ref_cast::RefCast;

use std::collections::VecDeque;

pub type Messages = HashSet<MessageId>;
pub type Edges = HashSet<Edge>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub type Indexations = Vec<(HashedIndex, MessageId)>;
pub type UnconfirmedMessages = Vec<(MilestoneIndex, UnconfirmedMessage)>;
pub type Receipts = Vec<(MilestoneIndex, Receipt)>;

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from_parent: MessageId,
    pub to_child: MessageId,
}

/// Collects all prunable nodes/vertices and edges of the Tangle for the specified index.
pub async fn collect_confirmed_data<B: StorageBackend>(
    tangle: &MsTangle<B>,
    target_index: MilestoneIndex,
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

    // We get us a clone of the current SEP set. We are the only ones that make changes to that tangle state, so we can
    // be sure it can't be invalidated in the meantime while we do the past-cone traversal.
    let old_seps = tangle.get_all_solid_entry_points().await;

    // We use a `VecDeque` here to traverse the past-cone in breadth-first manner. This gives use the necessary
    // guarantee that we "see" the children before any of their parents. We can use this guarantee to decide whether
    // within one past-cone of a milestone a message would be a redundant SEP, because all of its children/approvers are
    // already confirmed by the same or another milestone, and hence can be ignored.
    let mut parents: VecDeque<_> = vec![target_id].into_iter().collect();

    while let Some(current_id) = parents.pop_front() {
        // Skip conditions that will make the traversal terminate eventually:
        // (1) already processed that message (since children share parents)
        // (2) hit an old SEP (reached the "bottom" of the cone)
        if collected_messages.contains(&current_id) || old_seps.contains_key(SolidEntryPoint::ref_cast(&current_id)) {
            continue;
        } else {
            // // The message we are looking at was confirmed by the root index.
            // debug_assert_eq!(
            //     root_index,
            //     tangle
            //         .get_metadata(&current_id)
            //         .await
            //         // `unwrap` should be safe since we traverse the past of a milestone that has been solidified!
            //         .unwrap()
            //         .milestone_index()
            //         // `unwrap` should be safe since we traverse the past of a confirmed milestone!
            //         .unwrap()
            // );

            // We must be able to get its parents (unless the db is corrupt)
            if let Some((maybe_payload, current_parents)) = tangle.get(&current_id).await.map(|current| {
                (
                    current.payload().clone(),
                    current.parents().iter().copied().collect::<Vec<_>>(),
                )
            })
            // `Unwrap` should be safe since we traverse the past of a confirmed milestone!
            // .unwrap();
            {
                // Collect possible indexation payloads.
                if let Some(indexation) = unwrap_indexation(maybe_payload) {
                    let hashed_index = indexation.hash();

                    indexations.push((hashed_index, current_id));
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
                error!(
                    "Fetching message data for confirmed_message {} failed. This is a bug!",
                    current_id
                );
            }

            let _ = collected_messages.insert(current_id);

            // We only add this as a new SEP if it has at least one unconfirmed aprover.
            // `Unwrap` should be safe, because the current message has been confirmed, and hence must have approvers.
            if let Some(approvers) = tangle.get_children(&current_id).await {
                // If all approvers are part of the `new_seps` list already, then we can assume that this potential SEP
                // is redundant. Since we traverse the past-cone breadth-first we can be sure that
                // approvers are visited before their respective approvees, and the following skip
                // condition is triggered often. This allows us to not having to fetch the metadata for
                // all of its approvers from the Tangle.
                //
                // This is an efficient method to find the "surface" of a confirmed past-cone, that is all confirmed
                // messages, that have at least one child not confirmed by this milestone.
                if approvers
                    .iter()
                    .all(|approver_id| new_seps.contains_key(SolidEntryPoint::ref_cast(approver_id)))
                {
                    continue;
                }
            } else {
                error!(
                    "Fetching approvers for confirmed_message {} failed. This is a bug!",
                    current_id
                );
            }

            // WE don't care for the actual milestone index that actually confirmed that SEP, so we forget about it, and
            // just set it to the `target_index`.
            let _ = new_seps.insert(current_id.into(), target_index);
        }
    }

    Ok((collected_messages, edges, new_seps, indexations))
}

pub async fn collect_unconfirmed_data<B: StorageBackend>(
    storage: &StorageHooks<B>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(UnconfirmedMessages, Edges, Indexations), Error> {
    let mut unconfirmed = UnconfirmedMessages::default();
    let mut edges = Edges::default();
    let mut indexations = Indexations::default();

    for index in *start_index..=*target_index {
        collect_unconfirmed_data_by_index(storage, index.into(), &mut unconfirmed, &mut edges, &mut indexations)
            .await?;
    }

    Ok((unconfirmed, edges, indexations))
}

/// Get the unconfirmed/unreferenced messages.
async fn collect_unconfirmed_data_by_index<B: StorageBackend>(
    storage: &StorageHooks<B>,
    index: MilestoneIndex,
    unconfirmed: &mut UnconfirmedMessages,
    edges: &mut Edges,
    indexations: &mut Indexations,
) -> Result<(), Error> {
    let fetched = Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(&***storage, &index)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?
        // TODO: explain why there are always unconfirmed messages per each milestone
        .unwrap();

    if fetched.is_empty() {
        return Ok(());
    }

    for unconfirmed_message_id in fetched.iter().map(|msg| msg.message_id()) {
        if let Some((maybe_payload, parents)) = Fetch::<MessageId, Message>::fetch(&***storage, &unconfirmed_message_id)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?
            .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
        // TODO: explain why that `unwrap` is safe? Why do we know that the `Fetch` must find that message?
        // .unwrap();
        {
            // Collect possible indexation payloads
            if let Some(indexation) = unwrap_indexation(maybe_payload) {
                let hashed_index = indexation.hash();
                let message_id = *unconfirmed_message_id;

                indexations.push((hashed_index, message_id));
            }

            // Collect edges
            for parent in parents.iter() {
                edges.insert(Edge {
                    from_parent: *parent,
                    to_child: *unconfirmed_message_id,
                });
            }
        } else {
            error!(
                "Fetching message data for unconfirmed_message {} failed. This is a bug!",
                unconfirmed_message_id
            );
        }
    }

    // Unconfirmed messages associated with their milestone index, which we need for the batch operation.
    unconfirmed.extend(std::iter::repeat(index).zip(fetched.into_iter()));

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

pub async fn collect_receipts<B: StorageBackend>(
    storage: &StorageHooks<B>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<Receipts, Error> {
    let mut receipts = Receipts::default();

    for index in *start_index..=*target_index {
        let fetched = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(&***storage, &index.into())
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
