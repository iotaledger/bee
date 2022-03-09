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
    address::Address,
    input::Input,
    milestone::MilestoneIndex,
    output::{
        AliasId, AliasOutput, BasicOutput, ChainId, FeatureBlock, FoundryOutput, NftId, NftOutput, Output, OutputId,
        TokenId, UnlockCondition,
    },
    payload::{
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload,
    },
    signature::Signature,
    unlock_block::{UnlockBlock, UnlockBlocks},
    Message, MessageId,
};
use bee_tangle::{ConflictReason, Tangle};

use crypto::hashes::blake2b::Blake2b256;
use primitive_types::U256;

use std::collections::{HashMap, HashSet};

struct ValidationContext<'a> {
    essence: &'a RegularTransactionEssence,
    essence_hash: [u8; 32],
    unlock_blocks: &'a UnlockBlocks,
    milestone_index: MilestoneIndex,
    milestone_timestamp: u64,
    input_amount: u64,
    input_native_tokens: HashMap<TokenId, U256>,
    input_chains: HashMap<ChainId, Output>,
    output_amount: u64,
    output_native_tokens: HashMap<TokenId, U256>,
    output_chains: HashMap<ChainId, Output>,
    consumed_outputs: Vec<(OutputId, CreatedOutput)>,
    unlocked_addresses: HashSet<Address>,
}

impl<'a> ValidationContext<'a> {
    fn new(
        essence: &'a RegularTransactionEssence,
        unlock_blocks: &'a UnlockBlocks,
        milestone_index: MilestoneIndex,
        milestone_timestamp: u64,
    ) -> Self {
        Self {
            essence,
            unlock_blocks,
            essence_hash: TransactionEssence::from(essence.clone()).hash(),
            milestone_index,
            milestone_timestamp,
            input_amount: 0,
            input_native_tokens: HashMap::<TokenId, U256>::new(),
            input_chains: HashMap::new(),
            output_amount: 0,
            output_native_tokens: HashMap::<TokenId, U256>::new(),
            output_chains: HashMap::new(),
            consumed_outputs: Vec::with_capacity(essence.inputs().len()),
            unlocked_addresses: HashSet::new(),
        }
    }
}

fn check_input_unlock_conditions(
    unlock_conditions: &[UnlockCondition],
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    for unlock_condition in unlock_conditions {
        match unlock_condition {
            UnlockCondition::Address(_) => {
                todo!()
            }
            UnlockCondition::StorageDepositReturn(_) => {
                todo!()
            }
            UnlockCondition::Timelock(timelock) => {
                if *timelock.milestone_index() != 0 && context.milestone_index < timelock.milestone_index() {
                    return Err(ConflictReason::TimelockMilestoneIndex);
                }
                if timelock.timestamp() != 0 && context.milestone_timestamp < timelock.timestamp() as u64 {
                    return Err(ConflictReason::TimelockUnix);
                }
            }
            UnlockCondition::Expiration(_) => {
                todo!()
            }
            UnlockCondition::StateControllerAddress(_) => {
                todo!()
            }
            UnlockCondition::GovernorAddress(_) => {
                todo!()
            }
            UnlockCondition::ImmutableAliasAddress(_) => {
                todo!()
            }
        }
    }

    Ok(())
}

fn check_output_feature_blocks(
    feature_blocks: &[FeatureBlock],
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    for feature_block in feature_blocks {
        match feature_block {
            FeatureBlock::Sender(sender) => {
                if !context.unlocked_addresses.contains(sender.address()) {
                    return Err(ConflictReason::UnverifiedSender);
                }
            }
            FeatureBlock::Issuer(_) => {
                todo!()
            }
            _ => {}
        }
    }

    Ok(())
}

fn unlock_address(
    address: &Address,
    unlock_block: &UnlockBlock,
    context: &mut ValidationContext,
) -> Result<(), ConflictReason> {
    match (address, unlock_block) {
        (Address::Ed25519(ed25519_address), UnlockBlock::Signature(unlock_block)) => {
            if let Signature::Ed25519(signature) = unlock_block.signature() {
                if ed25519_address.verify(&context.essence_hash, signature).is_err() {
                    return Err(ConflictReason::InvalidSignature);
                }
            } else {
                return Err(ConflictReason::InvalidSignature);
            }
        }
        (Address::Alias(alias_address), UnlockBlock::Alias(unlock_block)) => {
            // SAFETY: indexing is fine as it is already syntactically verified that indexes reference below.
            if let Output::Alias(alias_output) = context.consumed_outputs[unlock_block.index() as usize].1.inner() {
                if alias_output.alias_id() != alias_address.alias_id() {
                    return Err(ConflictReason::IncorrectUnlockMethod);
                }
            } else {
                return Err(ConflictReason::IncorrectUnlockMethod);
            }
        }
        (Address::Nft(nft_address), UnlockBlock::Nft(unlock_block)) => {
            // SAFETY: indexing is fine as it is already syntactically verified that indexes reference below.
            if let Output::Nft(nft_output) = context.consumed_outputs[unlock_block.index() as usize].1.inner() {
                if nft_output.nft_id() != nft_address.nft_id() {
                    return Err(ConflictReason::IncorrectUnlockMethod);
                }
            } else {
                return Err(ConflictReason::IncorrectUnlockMethod);
            }
        }
        _ => return Err(ConflictReason::IncorrectUnlockMethod),
    }

    context.unlocked_addresses.insert(*address);

    Ok(())
}

fn unlock_basic_output(
    output_id: &OutputId,
    output: &BasicOutput,
    unlock_block: &UnlockBlock,
    context: &mut ValidationContext,
) -> Result<(), ConflictReason> {
    unlock_address(output.address(), unlock_block, context)
}

fn unlock_alias_output(
    output_id: &OutputId,
    current_state: &AliasOutput,
    unlock_block: &UnlockBlock,
    context: &mut ValidationContext,
) -> Result<(), ConflictReason> {
    let alias_id = if current_state.alias_id().is_null() {
        AliasId::new(output_id.hash())
    } else {
        *current_state.alias_id()
    };

    // TODO
    let next_state: Option<&AliasOutput> = None;

    // The alias is transitioned.
    if let Some(next_state) = next_state {
        // State transition.
        if next_state.state_index() == current_state.state_index() + 1 {
            unlock_address(current_state.state_controller_address(), unlock_block, context)?;
        }
        // Governance transition.
        else if next_state.state_index() == current_state.state_index() {
            unlock_address(current_state.governor_address(), unlock_block, context)?;
        } else {
            // TODO Err non contiguous state increase
        }
    }
    // The alias is destroyed.
    else {
    }

    Ok(())
}

fn unlock_foundry_output(
    output_id: &OutputId,
    output: &FoundryOutput,
    unlock_block: &UnlockBlock,
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    Ok(())
}

fn unlock_nft_output(
    output_id: &OutputId,
    output: &NftOutput,
    unlock_block: &UnlockBlock,
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    let nft_id = if output.nft_id().is_null() {
        NftId::new(output_id.hash())
    } else {
        *output.nft_id()
    };

    Ok(())
}

fn apply_regular_essence<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction_id: &TransactionId,
    essence: &RegularTransactionEssence,
    unlock_blocks: &UnlockBlocks,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    let mut context = ValidationContext::new(
        essence,
        unlock_blocks,
        metadata.milestone_index,
        metadata.milestone_timestamp,
    );

    // TODO check inputs commitment.

    for (index, input) in essence.inputs().iter().enumerate() {
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

        // SAFETY: it is already known that there is the same amount of inputs and unlock blocks.
        let unlock_block = unlock_blocks.get(index).unwrap();

        let (amount, consumed_native_tokens) = match consumed_output.inner() {
            Output::Basic(output) => {
                if let Err(conflict) = unlock_basic_output(output_id, output, unlock_block, &mut context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            Output::Alias(output) => {
                if let Err(conflict) = unlock_alias_output(output_id, output, unlock_block, &mut context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            Output::Foundry(output) => {
                if let Err(conflict) = unlock_foundry_output(output_id, output, unlock_block, &context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            Output::Nft(output) => {
                if let Err(conflict) = unlock_nft_output(output_id, output, unlock_block, &context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            _ => return Err(Error::UnsupportedOutputKind(consumed_output.inner().kind())),
        };

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

        context.consumed_outputs.push((*output_id, consumed_output));
    }

    for created_output in essence.outputs() {
        // TODO also check feature blocks ?
        let (amount, created_native_tokens) = match created_output {
            // TODO chain constraints
            Output::Basic(output) => (output.amount(), output.native_tokens()),
            Output::Alias(output) => (output.amount(), output.native_tokens()),
            Output::Foundry(output) => (output.amount(), output.native_tokens()),
            Output::Nft(output) => (output.amount(), output.native_tokens()),
            _ => return Err(Error::UnsupportedOutputKind(created_output.kind())),
        };

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

    if context.input_amount != context.output_amount {
        return Ok(ConflictReason::CreatedConsumedAmountMismatch);
    }

    if context.input_native_tokens != context.output_native_tokens {
        return Ok(ConflictReason::CreatedConsumedNativeTokensAmountMismatch);

        // TODO The transaction is balanced in terms of native tokens, meaning the amount of native tokens present in
        // inputs equals to that of outputs. Otherwise, the foundry outputs controlling outstanding native token
        // balances must be present in the transaction. The validation of the foundry output(s) determines if
        // the outstanding balances are valid.
    }

    // TODO check chain constraints

    for (output_id, created_output) in context.consumed_outputs {
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
