// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{ConsumedOutput, CreatedOutput},
    workers::{
        consensus::{merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata},
        error::Error,
        storage::{self, StorageBackend},
    },
};

use bee_message::{
    input::Input,
    output::{Output, OutputId},
    payload::{
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload,
    },
    semantic::{ConflictReason, ValidationContext},
    unlock_block::UnlockBlocks,
    Message, MessageId,
};
use bee_tangle::Tangle;

use crypto::hashes::blake2b::Blake2b256;

use std::collections::HashSet;

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

    let mut context = ValidationContext::new(
        transaction_id,
        essence,
        inputs.iter().map(|(output_id, input)| (output_id, *input)),
        unlock_blocks,
        metadata.milestone_index,
        metadata.milestone_timestamp,
    );

    // Validation of the inputs commitment.
    if context.essence.inputs_commitment() != &context.inputs_commitment {
        return Ok(ConflictReason::InputsCommitmentsMismatch);
    }

    // Validation of inputs.
    for ((output_id, consumed_output), unlock_block) in inputs.iter().zip(unlock_blocks.iter()) {
        let (conflict, amount, consumed_native_tokens, unlock_conditions) = match consumed_output {
            Output::Basic(output) => (
                output.unlock(output_id, unlock_block, &inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Alias(output) => (
                output.unlock(output_id, unlock_block, &inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Foundry(output) => (
                output.unlock(output_id, unlock_block, &inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            Output::Nft(output) => (
                output.unlock(output_id, unlock_block, &inputs, &mut context),
                output.amount(),
                output.native_tokens(),
                output.unlock_conditions(),
            ),
            _ => return Err(Error::UnsupportedOutputKind(consumed_output.kind())),
        };

        if let Err(conflict) = conflict {
            return Ok(conflict);
        }

        // TODO maybe put in unlocks methods.
        if let Some(timelock) = unlock_conditions.timelock() {
            if *timelock.milestone_index() != 0 && context.milestone_index < timelock.milestone_index() {
                return Ok(ConflictReason::TimelockMilestoneIndex);
            }
            if timelock.timestamp() != 0 && context.milestone_timestamp < timelock.timestamp() as u64 {
                return Ok(ConflictReason::TimelockUnix);
            }
        }

        if let Some(storage_deposit_return) = unlock_conditions.storage_deposit_return() {
            let amount = context
                .storage_deposit_returns
                .entry(*storage_deposit_return.return_address())
                .or_default();

            *amount = amount
                .checked_add(storage_deposit_return.amount())
                .ok_or(Error::StorageDepositReturnOverflow)?;
        }

        context.input_amount = context
            .input_amount
            .checked_add(amount)
            .ok_or(Error::ConsumedAmountOverflow)?;

        for native_token in consumed_native_tokens.iter() {
            let native_token_amount = context.input_native_tokens.entry(*native_token.token_id()).or_default();

            *native_token_amount = native_token_amount
                .checked_add(*native_token.amount())
                .ok_or(Error::ConsumedNativeTokensAmountOverflow)?;
        }
    }

    // Validation of outputs.
    for created_output in essence.outputs() {
        let (amount, created_native_tokens, feature_blocks) = match created_output {
            Output::Basic(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            Output::Alias(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            Output::Foundry(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            Output::Nft(output) => (output.amount(), output.native_tokens(), output.feature_blocks()),
            _ => return Err(Error::UnsupportedOutputKind(created_output.kind())),
        };

        if let Some(sender) = feature_blocks.sender() {
            if !context.unlocked_addresses.contains(sender.address()) {
                return Ok(ConflictReason::UnverifiedSender);
            }
        }

        context.output_amount = context
            .output_amount
            .checked_sub(amount)
            .ok_or(Error::CreatedAmountOverflow)?;

        for native_token in created_native_tokens.iter() {
            let native_token_amount = *context
                .output_native_tokens
                .entry(*native_token.token_id())
                .or_default();

            native_token_amount
                .checked_sub(*native_token.amount())
                .ok_or(Error::CreatedNativeTokensAmountOverflow)?;
        }
    }

    // Validation of storage deposit returns.
    for (return_address, return_amount) in context.storage_deposit_returns.iter() {
        if !context.essence.outputs().iter().any(|output| {
            if let Output::Basic(output) = output {
                output.address() == return_address
                    && output.amount() == *return_amount
                    && output.unlock_conditions().len() == 1
                    && output.feature_blocks().len() == 0
            } else {
                false
            }
        }) {
            return Ok(ConflictReason::StorageDepositReturnMismatch);
        }
    }

    // Validation of amounts.
    if context.input_amount != context.output_amount {
        return Ok(ConflictReason::CreatedConsumedAmountMismatch);
    }

    // Validation of native tokens.
    if context.input_native_tokens != context.output_native_tokens {
        return Ok(ConflictReason::CreatedConsumedNativeTokensAmountMismatch);
    }

    // Validation of state creations and transitions.
    for (chain_id, current_state) in context.input_chains.iter() {
        if Output::state_transition(
            Some(current_state),
            context.output_chains.get(chain_id).map(core::ops::Deref::deref),
            &context,
        )
        .is_err()
        {
            return Ok(ConflictReason::SemanticValidationFailed);
        }
    }

    // Validation of state destructions.
    for (chain_id, next_state) in context.output_chains.iter() {
        if context.input_chains.get(chain_id).is_none()
            && Output::state_transition(None, Some(next_state), &context).is_err()
        {
            return Ok(ConflictReason::SemanticValidationFailed);
        }
    }

    // Explicitly dropping the context to release the borrow of inputs and avoid cloning in the loop below.
    drop(context);

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
