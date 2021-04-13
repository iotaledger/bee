// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::consensus::StorageBackend;

use bee_message::prelude::{Essence, HashedIndex, IndexationPayload, Message, MessageId, MilestoneIndex, Payload};
use bee_storage::access::Fetch;
use bee_tangle::{
    ms_tangle::StorageHooks, solid_entry_point::SolidEntryPoint, unconfirmed_message::UnconfirmedMessage, MsTangle,
};

use hashbrown::{HashMap, HashSet};
use ref_cast::RefCast;

use std::collections::VecDeque;

pub type Messages = HashSet<MessageId>;
pub type Edges = HashSet<Edge>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub type Indexations = Vec<(HashedIndex, MessageId)>;
pub type UnconfirmedMessages = Vec<UnconfirmedMessage>;

/// Collects all prunable nodes/vertices and edges of the Tangle for the specified index.
pub async fn collect_confirmed_data<B: StorageBackend>(
    tangle: &MsTangle<B>,
    current_index: MilestoneIndex,
    previous_seps: &HashSet<MessageId>,
) -> Result<(Messages, Edges, Seps, Indexations), Error> {
    let mut messages = Messages::default();
    let mut edges = Edges::default();
    let mut seps = Seps::default();
    let mut indexations = Indexations::default();

    let current_id = tangle
        .get_milestone_message_id(current_index)
        .await
        .ok_or(Error::MilestoneNotFoundInTangle(*current_index))?;

    let mut parents: VecDeque<_> = vec![current_id].into_iter().collect();

    while let Some(current_id) = parents.pop_front() {
        // Stop conditions:
        // (1) already seen
        // (2) SEP
        if messages.contains(&current_id)
            || previous_seps.contains(&current_id)
            || tangle.is_solid_entry_point(&current_id).await
        {
            continue;
        } else {
            let current_milestone_index = tangle
                .get_metadata(&current_id)
                .await
                // `unwrap` should be safe since we traverse the past of a milestone that has been solidified!
                .unwrap()
                .milestone_index()
                // `unwrap` should be safe since we traverse the past of a confirmed milestone!
                .unwrap();

            let (maybe_payload, current_parents) = tangle
                .get(&current_id)
                .await
                .map(|current| {
                    (
                        current.payload().clone(),
                        current.parents().iter().copied().collect::<Vec<_>>(),
                    )
                })
                // `Unwrap` should be safe since we traverse the past of a confirmed milestone!
                .unwrap();

            // Collect possible indexation payloads
            if let Some(indexation) = unwrap_indexation(maybe_payload) {
                let hashed_index = indexation.hash();

                indexations.push((hashed_index, current_id));
            }

            // Collect edges
            for parent_id in &current_parents {
                let _ = edges.insert(Edge {
                    from: *parent_id,
                    to: current_id,
                });
            }

            parents.append(&mut current_parents.into_iter().collect());

            let _ = messages.insert(current_id);

            // We only add this as a new SEP if it has at least one unconfirmed aprover.
            // `Unwrap` should be safe, because the current message has been confirmed, and hence must have approvers.
            let approvers = tangle.get_children(&current_id).await.unwrap();

            // If all approvers are part of the current SEP list, then we can assume this potential SEP is redundant.
            // Since we traverse the past-cone breadth-first we can be sure that approvers are visited before their
            // respective approvees, and the following `continue` is triggered often. This allows us to not having to
            // fetch the metadata for all of its approvers from the Tangle.
            if approvers
                .iter()
                .all(|approver_id| seps.contains_key(SolidEntryPoint::ref_cast(approver_id)))
            {
                continue;
            }

            let _ = seps.insert(current_id.into(), current_milestone_index);
        }
    }

    Ok((messages, edges, seps, indexations))
}

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from: MessageId,
    pub to: MessageId,
}

/// Get the unconfirmed/unreferenced messages.
pub async fn collect_unconfirmed_data<B: StorageBackend>(
    storage: &StorageHooks<B>,
    index: MilestoneIndex,
) -> Result<(UnconfirmedMessages, Edges, Indexations), Error> {
    let mut edges = Edges::default();
    let mut indexations = Indexations::default();

    let unconfirmed = Fetch::<MilestoneIndex, Vec<UnconfirmedMessage>>::fetch(&***storage, &index)
        .await
        .map_err(|e| Error::StorageError(Box::new(e)))?
        // TODO: explain why there are always unconfirmed messages per each milestone
        .unwrap();

    if unconfirmed.is_empty() {
        return Ok((unconfirmed, edges, indexations));
    }

    for unconfirmed_message_id in unconfirmed.iter().map(|msg| msg.message_id()) {
        let (maybe_payload, parents) = Fetch::<MessageId, Message>::fetch(&***storage, &unconfirmed_message_id)
            .await
            .map_err(|e| Error::StorageError(Box::new(e)))?
            .map(|m| (m.payload().clone(), m.parents().iter().copied().collect::<Vec<_>>()))
            // TODO: explain why that `unwrap` is safe? Why do we know that the `Fetch` must find that message?
            .unwrap();

        // Collect possible indexation payloads
        if let Some(indexation) = unwrap_indexation(maybe_payload) {
            let hashed_index = indexation.hash();
            let message_id = *unconfirmed_message_id;

            indexations.push((hashed_index, message_id));
        }

        // Collect edges
        for parent in parents.iter() {
            edges.insert(Edge {
                from: *parent,
                to: *unconfirmed_message_id,
            });
        }
    }

    Ok((unconfirmed, edges, indexations))
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

/// Collects unconfirmed messages by walking the future cones of all confirmed messages by a particular milestone.
#[allow(dead_code)]
pub async fn collect_unconfirmed_data_improved<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    _target_index: MilestoneIndex,
) -> Result<(Messages, Edges, Seps), Error> {
    // This is an alternative way of fetching unconfirmed messages, which could help reduce database operations.
    todo!()
}
