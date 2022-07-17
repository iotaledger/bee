// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use bee_block::{
    input::Input,
    output::{Output, OutputId},
    payload::{
        milestone::MerkleRoot,
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload,
    },
    semantic::{semantic_validation, ConflictReason, ValidationContext},
    unlock::Unlocks,
    Block, BlockId,
};
use bee_tangle::Tangle;
use crypto::hashes::blake2b::Blake2b256;

use crate::{
    consensus::{merkle_hasher::MerkleHasher, metadata::WhiteFlagMetadata},
    error::Error,
    storage::{self, StorageBackend},
    types::{ConsumedOutput, CreatedOutput},
};

fn apply_regular_essence<B: StorageBackend>(
    storage: &B,
    block_id: &BlockId,
    transaction_id: &TransactionId,
    essence: &RegularTransactionEssence,
    unlocks: &Unlocks,
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
        unlocks,
        metadata.milestone_timestamp,
    );

    let conflict = semantic_validation(context, &inputs, unlocks)?;

    if conflict != ConflictReason::None {
        return Ok(conflict);
    }

    for (output_id, created_output) in consumed_outputs {
        metadata.consumed_outputs.insert(
            output_id,
            (
                created_output,
                ConsumedOutput::new(*transaction_id, metadata.milestone_index, metadata.milestone_timestamp),
            ),
        );
    }

    for (index, output) in essence.outputs().iter().enumerate() {
        metadata.created_outputs.insert(
            OutputId::new(*transaction_id, index as u16)?,
            CreatedOutput::new(
                *block_id,
                metadata.milestone_index,
                metadata.milestone_timestamp,
                output.clone(),
            ),
        );
    }

    Ok(ConflictReason::None)
}

fn apply_transaction<B: StorageBackend>(
    storage: &B,
    block_id: &BlockId,
    transaction: &TransactionPayload,
    metadata: &mut WhiteFlagMetadata,
) -> Result<ConflictReason, Error> {
    match transaction.essence() {
        TransactionEssence::Regular(essence) => apply_regular_essence(
            storage,
            block_id,
            &transaction.id(),
            essence,
            transaction.unlocks(),
            metadata,
        ),
    }
}

fn apply_block<B: StorageBackend>(
    storage: &B,
    block_id: &BlockId,
    block: &Block,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    metadata.referenced_blocks.push(*block_id);

    match block.payload() {
        Some(Payload::Transaction(transaction)) => match apply_transaction(storage, block_id, transaction, metadata)? {
            ConflictReason::None => metadata.included_blocks.push(*block_id),
            conflict => metadata.excluded_conflicting_blocks.push((*block_id, conflict)),
        },
        Some(Payload::Milestone(milestone)) => {
            if let Some(previous_milestone_id) = metadata.previous_milestone_id {
                if previous_milestone_id == milestone.id() {
                    metadata.found_previous_milestone = true;
                }
            }
            metadata.excluded_no_transaction_blocks.push(*block_id);
        }
        _ => metadata.excluded_no_transaction_blocks.push(*block_id),
    }

    Ok(())
}

async fn traverse_past_cone<B: StorageBackend>(
    tangle: &Tangle<B>,
    storage: &B,
    mut block_ids: Vec<BlockId>,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    let mut visited = HashSet::new();

    while let Some(block_id) = block_ids.last() {
        if let Some((block, meta)) = tangle.get_block_and_metadata(block_id) {
            if meta.flags().is_referenced() {
                visited.insert(*block_id);
                block_ids.pop();
                continue;
            }

            if let Some(unvisited) = block.parents().iter().find(|p| !visited.contains(p)) {
                block_ids.push(*unvisited);
            } else {
                if !visited.contains(block_id) {
                    apply_block(storage, block_id, &block, metadata)?;
                    visited.insert(*block_id);
                }
                block_ids.pop();
            }
        } else if !tangle.is_solid_entry_point(block_id).await {
            return Err(Error::MissingBlock(*block_id));
        } else {
            visited.insert(*block_id);
            block_ids.pop();
        }
    }

    Ok(())
}

/// Computes the ledger state according to the White Flag method.
/// TIP: <https://github.com/iotaledger/tips/blob/main/tips/TIP-0002/tip-0002.md>
pub async fn white_flag<B: StorageBackend>(
    tangle: &Tangle<B>,
    storage: &B,
    block_ids: &[BlockId],
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    traverse_past_cone(tangle, storage, block_ids.iter().rev().copied().collect(), metadata).await?;

    // PANIC: unwrap is fine as Blake2b256 returns a hash of length MerkleRoot::LENGTH.
    metadata.inclusion_merkle_root = MerkleRoot::from(
        <[u8; MerkleRoot::LENGTH]>::try_from(MerkleHasher::<Blake2b256>::new().digest(&metadata.referenced_blocks))
            .unwrap(),
    );
    // PANIC: unwrap is fine as Blake2b256 returns a hash of length MerkleRoot::LENGTH.
    metadata.applied_merkle_root = MerkleRoot::from(
        <[u8; MerkleRoot::LENGTH]>::try_from(MerkleHasher::<Blake2b256>::new().digest(&metadata.included_blocks))
            .unwrap(),
    );

    if *metadata.milestone_index != 1 && metadata.previous_milestone_id.is_some() && !metadata.found_previous_milestone
    {
        return Err(Error::PreviousMilestoneNotFound);
    }

    if metadata.referenced_blocks.len()
        != metadata.excluded_no_transaction_blocks.len()
            + metadata.excluded_conflicting_blocks.len()
            + metadata.included_blocks.len()
    {
        return Err(Error::InvalidBlocksCount(
            metadata.referenced_blocks.len(),
            metadata.excluded_no_transaction_blocks.len(),
            metadata.excluded_conflicting_blocks.len(),
            metadata.included_blocks.len(),
        ));
    }

    Ok(())
}
