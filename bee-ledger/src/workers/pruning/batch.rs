// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use bee_common::packable::Packable;
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
    metadata::MessageMetadata, solid_entry_point::SolidEntryPoint, unreferenced_message::UnreferencedMessage,
};
use hashbrown::{HashMap, HashSet};
use ref_cast::RefCast;

use crate::{
    types::{ConsumedOutput, CreatedOutput, OutputDiff, Receipt},
    workers::{
        pruning::{
            error::PruningError,
            metrics::{ConfirmedDataPruningMetrics, MilestoneDataPruningMetrics, UnconfirmedDataPruningMetrics},
        },
        storage::StorageBackend,
    },
};

pub(crate) type Messages = HashSet<MessageId>;
pub(crate) type ApproverCache = HashMap<MessageId, MilestoneIndex>;
pub(crate) type Seps = HashMap<SolidEntryPoint, MilestoneIndex>;
pub(crate) type ByteLength = usize;

#[derive(Eq, PartialEq, Hash)]
pub(crate) struct Edge {
    pub(crate) from_parent: MessageId,
    pub(crate) to_child: MessageId,
}

pub(crate) fn batch_prunable_confirmed_data<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    current_seps: &Seps,
) -> Result<(Seps, ConfirmedDataPruningMetrics, ByteLength), PruningError> {
    // We keep a list of already visited messages.
    let mut visited = Messages::with_capacity(512);
    // We keep a cache of approvers to prevent fetch the same data from the storage more than once.
    let mut approver_cache = ApproverCache::with_capacity(512);
    // We collect new SEPs during the traversal, and return them as a result of this function.
    let mut new_seps = Seps::with_capacity(512);
    // We collect stats during the traversal, and return them as a result of this function.
    let mut metrics = ConfirmedDataPruningMetrics::default();
    // We count the number of bytes pruned from the storage.
    let mut byte_length = 0usize;

    // Get the `MessageId` of the milestone we are about to prune from the storage.
    let milestone_to_prune = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &prune_index)
        .map_err(|e| PruningError::Storage(Box::new(e)))?
        .ok_or(PruningError::MissingMilestone(prune_index))?;

    byte_length += prune_index.packed_len();
    byte_length += milestone_to_prune.packed_len();

    let prune_id = *milestone_to_prune.message_id();

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
            .map_err(|e| PruningError::Storage(Box::new(e)))?
            .ok_or(PruningError::MissingMessage(message_id))
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

            byte_length += prune_indexation_data::<_, BY_SIZE>(storage, batch, &(padded_index, message_id))?;

            metrics.prunable_indexations += 1;
        }

        // Delete its edges.
        let parents = msg.parents();
        for parent_id in parents.iter() {
            byte_length += prune_edge::<_, BY_SIZE>(storage, batch, &(*parent_id, message_id))?;

            metrics.prunable_edges += 1;
        }

        // Add its parents to the queue of yet to traverse messages.
        to_visit.extend(msg.into_parents().iter());

        // Remember that we've seen this message already.
        visited.insert(message_id);

        // Delete its associated data.
        byte_length += prune_message_and_metadata::<_, BY_SIZE>(storage, batch, &message_id)?;

        // ---
        // Everything that follows is required to decide whether this message's id should be kept as a solid entry
        // point. We keep the set of SEPs minimal by checking whether there are still messages in future
        // milestone cones (beyond the current target index) that are referencing the currently processed
        // message (similar to a garbage collector we remove objects only if nothing is referencing it anymore).
        // ---

        // Fetch its approvers from the storage.
        let approvers = Fetch::<MessageId, Vec<MessageId>>::fetch(storage, &message_id)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
            .ok_or(PruningError::MissingApprovers(message_id))?;

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
                    .map_err(|e| PruningError::Storage(Box::new(e)))?
                    .ok_or(PruningError::MissingMetadata(unvisited_id))?;

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

    Ok((new_seps, metrics, byte_length))
}

pub(crate) fn batch_prunable_unconfirmed_data<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
) -> Result<(ByteLength, UnconfirmedDataPruningMetrics), PruningError> {
    let mut byte_length = 0usize;
    let mut metrics = UnconfirmedDataPruningMetrics::default();

    let unconf_msgs = match Fetch::<MilestoneIndex, Vec<UnreferencedMessage>>::fetch(storage, &prune_index)
        .map_err(|e| PruningError::Storage(Box::new(e)))?
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
        match Fetch::<MessageId, MessageMetadata>::fetch(storage, unconf_msg_id)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
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
        match Fetch::<MessageId, Message>::fetch(storage, unconf_msg_id)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
        {
            Some(msg) => {
                let payload = msg.payload().as_ref();
                let parents = msg.parents();

                // Add message data to the delete batch.
                byte_length += prune_message_and_metadata::<_, BY_SIZE>(storage, batch, unconf_msg_id)?;

                log::trace!("Pruned unconfirmed msg {} at {}.", unconf_msg_id, prune_index);

                if let Some(indexation) = unwrap_indexation(payload) {
                    let padded_index = indexation.padded_index();
                    let message_id = *unconf_msg_id;

                    // Add prunable indexations to the delete batch.
                    byte_length += prune_indexation_data::<_, BY_SIZE>(storage, batch, &(padded_index, message_id))?;

                    metrics.prunable_indexations += 1;
                }

                // Add prunable edges to the delete batch.
                for parent in parents.iter() {
                    byte_length += prune_edge::<_, BY_SIZE>(storage, batch, &(*parent, *unconf_msg_id))?;

                    metrics.prunable_edges += 1;
                }
            }
            None => {
                metrics.already_pruned += 1;
                continue 'next_unconf_msg;
            }
        }

        byte_length += prune_unreferenced_message::<_, BY_SIZE>(storage, batch, prune_index, (*unconf_msg_id).into())?;

        metrics.prunable_messages += 1;
    }

    Ok((byte_length, metrics))
}

pub(crate) fn prune_milestone_data<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    should_prune_receipts: bool,
) -> Result<(ByteLength, MilestoneDataPruningMetrics), PruningError> {
    let mut byte_length = 0usize;
    let mut metrics = MilestoneDataPruningMetrics::default();

    byte_length += prune_milestone::<_, BY_SIZE>(storage, batch, prune_index)?;
    byte_length += prune_output_diff::<_, BY_SIZE>(storage, batch, prune_index)?;

    if should_prune_receipts {
        let (num_receipts, num_bytes) = prune_receipts::<_, BY_SIZE>(storage, batch, prune_index)?;

        metrics.receipts = num_receipts;
        byte_length += num_bytes;
    }

    Ok((byte_length, metrics))
}

fn prune_message_and_metadata<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    message_id: &MessageId,
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if BY_SIZE {
        let msg = Fetch::<MessageId, Message>::fetch(storage, message_id)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
            .unwrap();
        byte_length += msg.packed_len();

        let md = Fetch::<MessageId, MessageMetadata>::fetch(storage, &message_id)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
            .unwrap();
        byte_length += md.packed_len();
    }

    Batch::<MessageId, Message>::batch_delete(storage, batch, message_id)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Batch::<MessageId, MessageMetadata>::batch_delete(storage, batch, message_id)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_edge<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    edge: &(MessageId, MessageId),
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if BY_SIZE {
        byte_length += edge.0.packed_len() + edge.1.packed_len();
    }

    Batch::<(MessageId, MessageId), ()>::batch_delete(storage, batch, edge)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_indexation_data<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    index_message_id: &(PaddedIndex, MessageId),
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if BY_SIZE {
        byte_length += index_message_id.0.packed_len() + index_message_id.1.packed_len();
    }

    Batch::<(PaddedIndex, MessageId), ()>::batch_delete(storage, batch, index_message_id)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_unreferenced_message<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    prune_index: MilestoneIndex,
    unreferenced_message: UnreferencedMessage,
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if BY_SIZE {
        byte_length += prune_index.packed_len() + unreferenced_message.packed_len();
    }

    Batch::<(MilestoneIndex, UnreferencedMessage), ()>::batch_delete(
        storage,
        batch,
        &(prune_index, unreferenced_message),
    )
    .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_milestone<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if BY_SIZE {
        let ms = Fetch::<MilestoneIndex, Milestone>::fetch(storage, &index)
            .map_err(|e| PruningError::Storage(Box::new(e)))?
            .unwrap();
        byte_length += ms.packed_len();
    }

    Batch::<MilestoneIndex, Milestone>::batch_delete(storage, batch, &index)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_output_diff<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<ByteLength, PruningError> {
    let mut byte_length = 0usize;

    if let Some(output_diff) =
        Fetch::<MilestoneIndex, OutputDiff>::fetch(storage, &index).map_err(|e| PruningError::Storage(Box::new(e)))?
    {
        byte_length += index.packed_len();
        byte_length += output_diff.packed_len();

        for consumed_output_id in output_diff.consumed_outputs() {
            if BY_SIZE {
                let consumed_output = Fetch::<OutputId, ConsumedOutput>::fetch(storage, consumed_output_id)
                    .map_err(|e| PruningError::Storage(Box::new(e)))?
                    .unwrap();

                byte_length += consumed_output_id.packed_len();
                byte_length += consumed_output.packed_len();

                let created_output = Fetch::<OutputId, CreatedOutput>::fetch(storage, consumed_output_id)
                    .map_err(|e| PruningError::Storage(Box::new(e)))?
                    .unwrap();

                byte_length += consumed_output_id.packed_len();
                byte_length += created_output.packed_len();
            }

            Batch::<OutputId, ConsumedOutput>::batch_delete(storage, batch, consumed_output_id)
                .map_err(|e| PruningError::Storage(Box::new(e)))?;

            Batch::<OutputId, CreatedOutput>::batch_delete(storage, batch, consumed_output_id)
                .map_err(|e| PruningError::Storage(Box::new(e)))?;
        }

        if let Some(_treasury_diff) = output_diff.treasury_diff() {
            // TODO
        }
    }

    Batch::<MilestoneIndex, OutputDiff>::batch_delete(storage, batch, &index)
        .map_err(|e| PruningError::Storage(Box::new(e)))?;

    Ok(byte_length)
}

fn prune_receipts<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    index: MilestoneIndex,
) -> Result<(usize, ByteLength), PruningError> {
    let mut byte_length = 0usize;

    let receipts = Fetch::<MilestoneIndex, Vec<Receipt>>::fetch(storage, &index)
        .map_err(|e| PruningError::Storage(Box::new(e)))?
        // Fine since Fetch of a Vec<_> always returns Some(Vec<_>).
        .unwrap();

    let mut num = 0;
    for receipt in receipts.into_iter() {
        if BY_SIZE {
            byte_length += index.packed_len();
            byte_length += receipt.packed_len();
        }

        Batch::<(MilestoneIndex, Receipt), ()>::batch_delete(storage, batch, &(index, receipt))
            .map_err(|e| PruningError::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok((num, byte_length))
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
fn prune_seps<S: StorageBackend, const BY_SIZE: bool>(
    storage: &S,
    batch: &mut S::Batch,
    seps: &[SolidEntryPoint],
) -> Result<usize, PruningError> {
    let mut num = 0;
    for sep in seps {
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_delete(storage, batch, sep)
            .map_err(|e| PruningError::Storage(Box::new(e)))?;

        num += 1;
    }

    Ok(num)
}
