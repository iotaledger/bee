// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::error::Error;

use crate::consensus::StorageBackend;

use bee_message::{
    payload::Payload,
    prelude::{HashedIndex, MilestoneIndex},
    MessageId,
};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle};

use hashbrown::{HashMap, HashSet};

pub type Messages = HashSet<MessageId>;
pub type Edges = HashSet<Edge>;
pub type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub type Indexes = Vec<(HashedIndex, MessageId)>;

/// Collects all prunable nodes/vertices and edges of the Tangle up to the `target_index`.
pub async fn collect_confirmed_data<B: StorageBackend>(
    tangle: &MsTangle<B>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
) -> Result<(Messages, Edges, Seps, Indexes), Error> {
    let mut messages = Messages::default();
    let mut edges = Edges::default();
    let mut seps = Seps::default();
    let mut indexes = Indexes::default();

    // We start collecting at the current pruning index.
    let start_index = *start_index;
    let target_index = *target_index;
    debug_assert!(target_index > start_index);

    for current_index in start_index..target_index {
        process_pastcone_by_index(
            tangle,
            current_index,
            &mut messages,
            &mut edges,
            &mut seps,
            &mut indexes,
        )
        .await?;
    }

    Ok((messages, edges, seps, indexes))
}

/// Processes the pastcone of a particular milestone given by its index.
async fn process_pastcone_by_index<B: StorageBackend>(
    tangle: &MsTangle<B>,
    current_index: u32,
    messages: &mut Messages,
    edges: &mut Edges,
    seps: &mut Seps,
    indexes: &mut Indexes,
) -> Result<(), Error> {
    let current_id = tangle
        .get_milestone_message_id(current_index.into())
        .await
        .ok_or(Error::MilestoneNotFoundInTangle(current_index))?;

    let mut parents = vec![current_id];

    while let Some(current_id) = parents.pop() {
        // Stop conditions:
        // (1) already seen
        // (2) SEP
        if messages.contains(&current_id) || tangle.is_solid_entry_point(&current_id).await {
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

            let (payload, mut current_parents) = tangle
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

            // Handle indexation payload
            if let Some(Payload::Indexation(payload)) = payload {
                let index = payload.hash();
                indexes.push((index, current_id));
            }

            for parent_id in &current_parents {
                let _ = edges.insert(Edge {
                    from: *parent_id,
                    to: current_id,
                });
            }

            parents.append(&mut current_parents);

            let _ = messages.insert(current_id);
            let _ = seps.insert(current_id.into(), current_milestone_index);
        }
    }

    Ok(())
}

#[derive(Eq, PartialEq, Hash)]
pub struct Edge {
    pub from: MessageId,
    pub to: MessageId,
}

/// Collects unconfirmed messages by walking the future cones of all confirmed messages by a particular milestone.
#[allow(dead_code)]
pub async fn collect_unconfirmed_data<B: StorageBackend>(
    _tangle: &MsTangle<B>,
    _target_index: MilestoneIndex,
) -> Result<(Messages, Edges, Seps), Error> {
    // This is an alternative way of fetching unconfirmed messages, which could help reduce database operations.
    todo!()
}
