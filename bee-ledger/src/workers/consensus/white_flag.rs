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
    output::{Output, OutputId},
    payload::{
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload,
    },
    unlock_block::{UnlockBlock, UnlockBlocks},
    Message, MessageId,
};
use bee_tangle::{ConflictReason, Tangle};

use crypto::hashes::blake2b::Blake2b256;

use std::collections::{HashMap, HashSet};

fn _verify_signature(_address: &Address, unlock_blocks: &UnlockBlocks, index: usize, _essence_hash: &[u8; 32]) -> bool {
    if let Some(UnlockBlock::Signature(_signature)) = unlock_blocks.get(index) {
        true
        // TODO
        // address.verify(essence_hash, signature).is_ok()
    } else {
        false
    }
}

fn apply_regular_essence<B: StorageBackend>(
    storage: &B,
    message_id: &MessageId,
    transaction_id: &TransactionId,
    essence: &RegularTransactionEssence,
    _unlock_blocks: &UnlockBlocks,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    let mut consumed_outputs = HashMap::with_capacity(essence.inputs().len());
    let balance_diffs = BalanceDiffs::new();
    let consumed_amount: u64 = 0;
    let created_amount: u64 = 0;

    for (_index, input) in essence.inputs().iter().enumerate() {
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

        let _essence_hash = TransactionEssence::from(essence.clone()).hash();

        match consumed_output.inner() {
            Output::Simple(_) => {
                // TODO
            }
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(consumed_output.inner().kind())),
            Output::Extended(_) => {
                // TODO
            }
            Output::Alias(_) => {
                // TODO
            }
            Output::Foundry(_) => {
                // TODO
            }
            Output::Nft(_) => {
                // TODO
            }
        }

        consumed_outputs.insert(*output_id, consumed_output);
    }

    for created_output in essence.outputs() {
        match created_output {
            Output::Simple(_) => {
                // TODO
            }
            Output::Treasury(_) => return Err(Error::UnsupportedOutputKind(created_output.kind())),
            Output::Extended(_) => {
                // TODO
            }
            Output::Alias(_) => {
                // TODO
            }
            Output::Foundry(_) => {
                // TODO
            }
            Output::Nft(_) => {
                // TODO
            }
        }
    }

    if created_amount != consumed_amount {
        return Ok(ConflictReason::InputOutputSumMismatch);
    }

    for (output_id, created_output) in consumed_outputs {
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

    metadata.balance_diffs.merge(balance_diffs)?;

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
