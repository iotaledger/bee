// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::ConflictReason,
    dust::{DUST_ALLOWANCE_DIVISOR, DUST_ALLOWANCE_MINIMUM},
    error::Error,
    metadata::WhiteFlagMetadata,
    model::{BalanceDiff, Output, Spent},
    storage::{self, StorageBackend},
};

use bee_message::{
    payload::{
        transaction::{self, Input, OutputId, TransactionPayload},
        Payload,
    },
    Message, MessageId,
};
use bee_runtime::node::Node;
use bee_tangle::MsTangle;

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

fn validate_transaction(
    transaction: &TransactionPayload,
    consumed_outputs: &HashMap<OutputId, Output>,
    balance_diff: &mut BalanceDiff,
) -> Result<ConflictReason, Error> {
    let mut created_amount: u64 = 0;
    let mut consumed_amount: u64 = 0;

    // TODO
    // The address type of the referenced UTXO must match the signature type contained in the corresponding Signature Unlock
    // Block. The Signature Unlock Blocks are valid, i.e. the signatures prove ownership over the addresses of the
    // referenced UTXOs.
    // TODO saturating ? Overflowing ? Checked ?
    // TODO also check against tota supply

    for created_output in transaction.essence().outputs() {
        match created_output {
            transaction::Output::SignatureLockedSingle(created_output) => {
                created_amount = created_amount.saturating_add(created_output.amount());
                balance_diff.balance_add(*created_output.address(), created_output.amount());
                if created_output.amount() < DUST_ALLOWANCE_MINIMUM {
                    balance_diff.dust_output_inc(*created_output.address());
                }
            }
            transaction::Output::SignatureLockedDustAllowance(created_output) => {
                created_amount = created_amount.saturating_add(created_output.amount());
                balance_diff.balance_add(*created_output.address(), created_output.amount());
                balance_diff.dust_allowance_add(*created_output.address(), created_output.amount());
            }
            _ => return Err(Error::UnsupportedOutputType),
        }
    }

    for (_index, (_, consumed_output)) in consumed_outputs.iter().enumerate() {
        match consumed_output.inner() {
            transaction::Output::SignatureLockedSingle(consumed_output) => {
                consumed_amount = consumed_amount.saturating_add(consumed_output.amount());
                balance_diff.balance_sub(*consumed_output.address(), consumed_output.amount());
                if consumed_output.amount() < DUST_ALLOWANCE_MINIMUM {
                    balance_diff.dust_output_dec(*consumed_output.address());
                }
            }
            transaction::Output::SignatureLockedDustAllowance(consumed_output) => {
                consumed_amount = consumed_amount.saturating_add(consumed_output.amount());
                balance_diff.balance_sub(*consumed_output.address(), consumed_output.amount());
                balance_diff.dust_allowance_sub(*consumed_output.address(), consumed_output.amount());
            }
            _ => return Err(Error::UnsupportedOutputType),
        }
    }

    if created_amount != consumed_amount {
        return Ok(ConflictReason::InputOutputSumMismatch);
    }

    Ok(ConflictReason::None)
}

#[inline]
async fn on_message<N: Node>(
    storage: &N::Backend,
    message_id: &MessageId,
    message: &Message,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    metadata.num_referenced_messages += 1;

    let transaction = if let Some(Payload::Transaction(transaction)) = message.payload() {
        transaction
    } else {
        metadata.excluded_no_transaction_messages.push(*message_id);
        return Ok(());
    };

    let transaction_id = transaction.id();
    let essence = transaction.essence();
    let mut spent_outputs = HashMap::with_capacity(essence.inputs().len());
    let mut conflict = ConflictReason::None;

    for input in essence.inputs() {
        if let Input::UTXO(utxo_input) = input {
            let output_id = utxo_input.output_id();

            if metadata.spent_outputs.contains_key(output_id) {
                conflict = ConflictReason::InputUTXOAlreadySpentInThisMilestone;
                break;
            }

            if let Some(output) = metadata.created_outputs.get(output_id).cloned() {
                spent_outputs.insert(*output_id, output);
                continue;
            }

            if let Some(output) = storage::fetch_output(storage.deref(), output_id).await? {
                if !storage::is_output_unspent(storage.deref(), output_id).await? {
                    conflict = ConflictReason::InputUTXOAlreadySpent;
                    break;
                }
                spent_outputs.insert(*output_id, output);
                continue;
            } else {
                conflict = ConflictReason::InputUTXONotFound;
                break;
            }
        } else {
            return Err(Error::UnsupportedInputType);
        };
    }

    if conflict != ConflictReason::None {
        metadata.excluded_conflicting_messages.push((*message_id, conflict));
        return Ok(());
    }

    let mut balance_diff = BalanceDiff::new();

    conflict = validate_transaction(&transaction, &spent_outputs, &mut balance_diff)?;

    if conflict != ConflictReason::None {
        metadata.excluded_conflicting_messages.push((*message_id, conflict));
        return Ok(());
    }

    for (address, entry) in balance_diff.iter() {
        // TODO conditionnally fetch ?
        let (mut dust_allowance, mut dust_output) = match storage::fetch_balance(storage.deref(), &address).await? {
            Some(balance) => (balance.dust_allowance as i64, balance.dust_output as i64),
            None => (0, 0),
        };

        if let Some(entry) = metadata.balance_diff.get(&address) {
            dust_allowance += entry.dust_allowance;
            dust_output += entry.dust_output;
        }

        if (dust_output as i64 + entry.dust_output) as u64
            > ((dust_allowance as i64 + entry.dust_allowance) as u64 / DUST_ALLOWANCE_DIVISOR)
        {
            metadata
                .excluded_conflicting_messages
                .push((*message_id, ConflictReason::InvalidDustAllowance));
            return Ok(());
        }
    }

    metadata.balance_diff.merge(balance_diff);

    // TODO store balances with diff

    for (index, output) in essence.outputs().iter().enumerate() {
        metadata.created_outputs.insert(
            // Unwrap is fine, the index is known to be valid.
            OutputId::new(transaction_id, index as u16).unwrap(),
            Output::new(*message_id, output.clone()),
        );
    }

    // TODO output ?
    for (output_id, _) in spent_outputs {
        metadata
            .spent_outputs
            .insert(output_id, Spent::new(transaction_id, metadata.index));
    }

    metadata.included_messages.push(*message_id);

    Ok(())
}

// TODO make it a tangle method ?
pub(crate) async fn traversal<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &N::Backend,
    mut messages_ids: Vec<MessageId>,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    let mut visited = HashSet::new();
    messages_ids = messages_ids.into_iter().rev().collect();

    // TODO Tangle get message AND meta at the same time

    while let Some(message_id) = messages_ids.last() {
        let meta = match tangle.get_metadata(message_id).await {
            Some(meta) => meta,
            None => {
                if !tangle.is_solid_entry_point(message_id) {
                    return Err(Error::MissingMessage(*message_id));
                } else {
                    visited.insert(*message_id);
                    messages_ids.pop();
                    continue;
                }
            }
        };

        if meta.flags().is_confirmed() {
            visited.insert(*message_id);
            messages_ids.pop();
            continue;
        }

        match tangle.get(message_id).await {
            Some(message) => {
                let parent1 = message.parent1();
                let parent2 = message.parent2();

                if visited.contains(parent1) && visited.contains(parent2) {
                    on_message::<N>(storage, message_id, &message, metadata).await?;
                    visited.insert(*message_id);
                    messages_ids.pop();
                } else if !visited.contains(parent1) {
                    messages_ids.push(*parent1);
                } else if !visited.contains(parent2) {
                    messages_ids.push(*parent2);
                }
            }
            None => {
                if !tangle.is_solid_entry_point(message_id) {
                    return Err(Error::MissingMessage(*message_id));
                } else {
                    visited.insert(*message_id);
                    messages_ids.pop();
                }
            }
        }
    }

    Ok(())
}
