// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{BalanceDiffs, ConsumedOutput, CreatedOutput},
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
    output::{AliasOutput, ExtendedOutput, FeatureBlock, NftOutput, Output, OutputId, TokenId},
    payload::{
        transaction::{Essence, RegularEssence, TransactionId, TransactionPayload},
        Payload,
    },
    unlock_block::{UnlockBlock, UnlockBlocks},
    Message, MessageId,
};
use bee_tangle::{ConflictReason, Tangle};

use crypto::hashes::blake2b::Blake2b256;
use primitive_types::U256;

use std::collections::{HashMap, HashSet};

struct ValidationContext {
    index: MilestoneIndex,
    timestamp: u64,
    amount: u64,
    native_tokens_amount: HashMap<TokenId, U256>,
    consumed_outputs: HashMap<OutputId, CreatedOutput>,
    balance_diffs: BalanceDiffs,
    essence_hash: [u8; 32],
    verified_addresses: HashSet<Address>,
}

impl ValidationContext {
    fn new(index: MilestoneIndex, timestamp: u64, essence: &RegularEssence) -> Self {
        Self {
            index,
            timestamp,
            amount: 0,
            native_tokens_amount: HashMap::<TokenId, U256>::new(),
            consumed_outputs: HashMap::with_capacity(essence.inputs().len()),
            balance_diffs: BalanceDiffs::new(),
            essence_hash: Essence::from(essence.clone()).hash(),
            verified_addresses: HashSet::new(),
        }
    }
}

fn check_input_feature_blocks(
    feature_blocks: &[FeatureBlock],
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    for feature_block in feature_blocks {
        match feature_block {
            FeatureBlock::Issuer(_) => {}
            FeatureBlock::DustDepositReturn(_) => {}
            FeatureBlock::TimelockMilestoneIndex(timelock) => {
                if context.index < timelock.index() {
                    return Err(ConflictReason::TimelockMilestoneIndex);
                }
            }
            FeatureBlock::TimelockUnix(timelock) => {
                if context.timestamp < timelock.timestamp() as u64 {
                    return Err(ConflictReason::TimelockUnix);
                }
            }
            FeatureBlock::ExpirationMilestoneIndex(_) => {}
            FeatureBlock::ExpirationUnix(_) => {}
            FeatureBlock::Indexation(_) => {
                // TODO map index-output
            }
            _ => {}
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
                if !context.verified_addresses.contains(sender.address()) {
                    return Err(ConflictReason::UnverifiedSender);
                }
            }
            FeatureBlock::Issuer(_) => {}
            FeatureBlock::DustDepositReturn(_) => {}
            FeatureBlock::TimelockMilestoneIndex(_) => {}
            FeatureBlock::TimelockUnix(_) => {}
            FeatureBlock::ExpirationMilestoneIndex(_) => {}
            FeatureBlock::ExpirationUnix(_) => {}
            FeatureBlock::Indexation(_) => {
                // TODO map index-output
            }
            _ => {}
        }
    }

    Ok(())
}

fn unlock_extended_output(
    output: &ExtendedOutput,
    unlock_blocks: &UnlockBlocks,
    index: usize,
    context: &mut ValidationContext,
) -> Result<(), ConflictReason> {
    // SAFETY: it is already known that there is the same amount of inputs and unlock blocks.
    if let UnlockBlock::Signature(signature) = unlock_blocks.get(index).unwrap() {
        if output.address().verify(&context.essence_hash, signature).is_ok() {
            // TODO another place where this should be done ?
            context.verified_addresses.insert(*output.address());
            check_input_feature_blocks(output.feature_blocks(), context)
        } else {
            Err(ConflictReason::InvalidSignature)
        }
    } else {
        todo!();
    }
}

fn unlock_alias_output(
    output: &AliasOutput,
    unlock_blocks: &UnlockBlocks,
    index: usize,
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    // SAFETY: it is already known that there is the same amount of inputs and unlock blocks.
    match unlock_blocks.get(index).unwrap() {
        UnlockBlock::Signature(_) => todo!(),
        UnlockBlock::Reference(_) => todo!(),
        UnlockBlock::Alias(_) => todo!(),
        UnlockBlock::Nft(_) => todo!(),
    }
}

fn unlock_nft_output(
    output: &NftOutput,
    unlock_blocks: &UnlockBlocks,
    index: usize,
    context: &ValidationContext,
) -> Result<(), ConflictReason> {
    // SAFETY: it is already known that there is the same amount of inputs and unlock blocks.
    match unlock_blocks.get(index).unwrap() {
        UnlockBlock::Signature(_) => todo!(),
        UnlockBlock::Reference(_) => todo!(),
        UnlockBlock::Alias(_) => todo!(),
        UnlockBlock::Nft(_) => todo!(),
    }
}

fn apply_regular_essence<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction_id: &TransactionId,
    essence: &RegularEssence,
    unlock_blocks: &UnlockBlocks,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    let mut context = ValidationContext::new(metadata.index, metadata.timestamp, &essence);

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
            Input::Treasury(_) => {
                return Err(Error::UnsupportedInputKind(input.kind()));
            }
        };

        let (amount, consumed_native_tokens) = match consumed_output.inner() {
            // Output::Simple(output) => {
            //     context.balance_diffs.amount_sub(*output.address(), output.amount())?;
            //
            //     if let Err(conflict) = unlock_input(output.address(), unlock_blocks, index, &essence_hash) {
            //         return Ok(conflict);
            //     }
            //
            //     (output.amount(), None)
            // }
            Output::Simple(_) => return Err(Error::UnsupportedOutputKind(consumed_output.inner().kind())),
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(consumed_output.inner().kind())),
            Output::Extended(output) => {
                if let Err(conflict) = unlock_extended_output(output, unlock_blocks, index, &mut context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            Output::Alias(output) => {
                if let Err(conflict) = unlock_alias_output(output, unlock_blocks, index, &context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
            Output::Foundry(output) => (output.amount(), output.native_tokens()),
            Output::Nft(output) => {
                if let Err(conflict) = unlock_nft_output(output, unlock_blocks, index, &context) {
                    return Ok(conflict);
                }

                (output.amount(), output.native_tokens())
            }
        };

        context.amount = context
            .amount
            .checked_add(amount)
            .ok_or(Error::ConsumedAmountOverflow)?;
        for native_token in consumed_native_tokens {
            let native_token_amount = context
                .native_tokens_amount
                .entry(*native_token.token_id())
                .or_default();
            *native_token_amount = native_token_amount
                .checked_add(*native_token.amount())
                .ok_or(Error::ConsumedNativeTokensAmountOverflow)?;
        }

        context.consumed_outputs.insert(*output_id, consumed_output);
    }

    for created_output in essence.outputs() {
        // TODO also check feature blocks ?
        let (amount, created_native_tokens) = match created_output {
            // Output::Simple(output) => {
            //     context.balance_diffs.amount_add(*output.address(), output.amount())?;
            //
            //     (output.amount(), None)
            // }
            Output::Simple(_) => return Err(Error::UnsupportedOutputKind(created_output.kind())),
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(created_output.kind())),
            Output::Extended(output) => (output.amount(), output.native_tokens()),
            Output::Alias(output) => (output.amount(), output.native_tokens()),
            Output::Foundry(output) => (output.amount(), output.native_tokens()),
            Output::Nft(output) => (output.amount(), output.native_tokens()),
        };

        context.amount = context.amount.checked_sub(amount).ok_or(Error::CreatedAmountOverflow)?;
        for native_token in created_native_tokens {
            let native_token_amount = *context
                .native_tokens_amount
                .entry(*native_token.token_id())
                .or_default();
            native_token_amount
                .checked_sub(*native_token.amount())
                .ok_or(Error::CreatedNativeTokensAmountOverflow)?;
        }
    }

    if context.amount != 0 {
        return Ok(ConflictReason::CreatedConsumedAmountMismatch);
    }

    for (_, amount) in context.native_tokens_amount {
        if !amount.is_zero() {
            return Ok(ConflictReason::CreatedConsumedNativeTokensAmountMismatch);
        }

        // TODO The transaction is balanced in terms of native tokens, meaning the amount of native tokens present in
        // inputs equals to that of outputs. Otherwise, the foundry outputs controlling outstanding native token
        // balances must be present in the transaction. The validation of the foundry output(s) determines if
        // the outstanding balances are valid.
    }

    // TODO check chain constraints

    for (output_id, created_output) in context.consumed_outputs {
        metadata.consumed_outputs.insert(
            output_id,
            (created_output, ConsumedOutput::new(*transaction_id, metadata.index)),
        );
    }

    for (index, output) in essence.outputs().iter().enumerate() {
        metadata.created_outputs.insert(
            OutputId::new(*transaction_id, index as u16)?,
            CreatedOutput::new(*message_id, output.clone()),
        );
    }

    metadata.balance_diffs.merge(context.balance_diffs)?;

    Ok(ConflictReason::None)
}

fn apply_transaction<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction: &TransactionPayload,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    match transaction.essence() {
        Essence::Regular(essence) => apply_regular_essence(
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
/// RFC: https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md
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

    let diff_sum = metadata.balance_diffs.iter().map(|(_, diff)| diff.amount()).sum();

    if diff_sum == 0 {
        Ok(())
    } else {
        Err(Error::NonZeroBalanceDiffSum(diff_sum))
    }
}
