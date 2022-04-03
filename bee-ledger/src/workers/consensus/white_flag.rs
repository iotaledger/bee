// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use bee_message::{
    input::Input,
    output::{Output, OutputId},
    payload::{
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload,
    },
    semantic::{semantic_validation, ConflictReason, ValidationContext},
    unlock_block::UnlockBlocks,
    Message, MessageId,
};
use bee_tangle::Tangle;
use crypto::hashes::blake2b::Blake2b256;

use crate::{
    types::{ConsumedOutput, CreatedOutput},
    workers::{
        consensus::{merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata},
        error::Error,
        storage::{self, StorageBackend},
    },
};

fn apply_regular_essence<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction_id: &TransactionId,
    essence: &RegularTransactionEssence,
    unlock_blocks: &UnlockBlocks,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    let mut consumed_outputs = Vec::<(OutputId, CreatedOutput)>::new();

    // Fetch inputs from the storage or from current milestone metadata.
    for input in essence.inputs().iter() {
        let (output_id, consumed_output) = match input {
            Input::Utxo(input) => {
                let output_id = input.output_id();

                if metadata.consumed_outputs.contains_key(output_id) {
                    return Ok(ConflictReason::InputUtxoAlreadySpentInThisMilestone);
                }

                if let Some(output) = metadata.created_outputs.get(output_id).cloned() {
                    (output_id, output)
                } else if let Some(output) = storage::fetch_output(storage, output_id)? {
                    if !storage::is_output_unspent(storage, output_id)? {
                        return Ok(ConflictReason::InputUtxoAlreadySpent);
                    }
                    (output_id, output)
                } else {
                    return Ok(ConflictReason::InputUtxoNotFound);
                }
            }
            _ => {
                return Err(Error::UnsupportedInputKind(input.kind()));
            }
        };

        consumed_outputs.push((*output_id, consumed_output));
    }

    let inputs: Vec<(OutputId, &Output)> = consumed_outputs
        .iter()
        .map(|(output_id, created_output)| (*output_id, created_output.inner()))
        .collect();

    let context = ValidationContext::new(
        transaction_id,
        essence,
        inputs.iter().map(|(output_id, input)| (output_id, *input)),
        unlock_blocks,
        metadata.milestone_index,
        metadata.milestone_timestamp,
    );

    let conflict = semantic_validation(context, &inputs, unlock_blocks)?;

    if conflict != ConflictReason::None {
        return Ok(conflict);
    }

    for (output_id, created_output) in consumed_outputs {
        metadata.consumed_outputs.insert(
            output_id,
            (
                created_output,
                ConsumedOutput::new(
                    *transaction_id,
                    metadata.milestone_index,
                    metadata.milestone_timestamp as u32,
                ),
            ),
        );
    }

    for (index, output) in essence.outputs().iter().enumerate() {
        metadata.created_outputs.insert(
            OutputId::new(*transaction_id, index as u16)?,
            CreatedOutput::new(
                *message_id,
                metadata.milestone_index,
                metadata.milestone_timestamp as u32,
                output.clone(),
            ),
        );
    }

    Ok(ConflictReason::None)
}

fn apply_transaction<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction: &TransactionPayload,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    match transaction.essence() {
        TransactionEssence::Regular(essence) => apply_regular_essence(
            storage,
            message_id,
            &transaction.id(),
            essence,
            transaction.unlock_blocks(),
            metadata,
        ),
    }
}

fn apply_message<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    message: &Message,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    metadata.referenced_messages += 1;

    match message.payload() {
        Some(Payload::Transaction(transaction)) => {
            match apply_transaction(storage, message_id, transaction, metadata)? {
                ConflictReason::None => metadata.included_messages.push(*message_id),
                conflict => metadata.excluded_conflicting_messages.push((*message_id, conflict)),
            }
        }
        _ => metadata.excluded_no_transaction_messages.push(*message_id),
    }

    Ok(())
}

async fn traverse_past_cone<B: StorageBackend>(
    tangle: &Tangle<B>,
    storage: &B,
    mut message_ids: Vec<MessageId>,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    let mut visited = HashSet::new();

    while let Some(message_id) = message_ids.last() {
        if let Some((message, meta)) = tangle
            .get_vertex(message_id)
            .await
            .as_ref()
            .and_then(|v| v.message_and_metadata().cloned())
        {
            if meta.flags().is_referenced() {
                visited.insert(*message_id);
                message_ids.pop();
                continue;
            }

            if let Some(unvisited) = message.parents().iter().find(|p| !visited.contains(p)) {
                message_ids.push(*unvisited);
            } else {
                apply_message(storage, message_id, &message, metadata)?;
                visited.insert(*message_id);
                message_ids.pop();
            }
        } else if !tangle.is_solid_entry_point(message_id).await {
            return Err(Error::MissingMessage(*message_id));
        } else {
            visited.insert(*message_id);
            message_ids.pop();
        }
    }

    Ok(())
}

/// Computes the ledger state according to the White Flag method.
/// TIP: <https://github.com/iotaledger/tips/blob/main/tips/TIP-0002/tip-0002.md>
pub async fn white_flag<B: StorageBackend>(
    tangle: &Tangle<B>,
    storage: &B,
    message_ids: &[MessageId],
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    traverse_past_cone(tangle, storage, message_ids.iter().rev().copied().collect(), metadata).await?;

    metadata.merkle_proof = MerkleHasher::<Blake2b256>::new().digest(&metadata.included_messages);

    if metadata.referenced_messages
        != metadata.excluded_no_transaction_messages.len()
            + metadata.excluded_conflicting_messages.len()
            + metadata.included_messages.len()
    {
        return Err(Error::InvalidMessagesCount(
            metadata.referenced_messages,
            metadata.excluded_no_transaction_messages.len(),
            metadata.excluded_conflicting_messages.len(),
            metadata.included_messages.len(),
        ));
    }

    Ok(())
}
