// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::ConflictReason,
    error::Error,
    metadata::WhiteFlagMetadata,
    model::{Output, Spent},
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

// TODO
// The address type of the referenced UTXO must match the signature type contained in the corresponding Signature Unlock
// Block. The Signature Unlock Blocks are valid, i.e. the signatures prove ownership over the addresses of the
// referenced UTXOs.

fn validate_transaction(
    transaction: &TransactionPayload,
    consumed_outputs: &HashMap<OutputId, Output>,
) -> Result<ConflictReason, Error> {
    let mut consumed_amount = 0;
    let mut created_amount = 0;

    for (_index, (_, consumed_output)) in consumed_outputs.iter().enumerate() {
        match consumed_output.inner() {
            transaction::Output::SignatureLockedSingle(consumed_output) => {
                consumed_amount += consumed_output.amount();
            }
            transaction::Output::SignatureLockedDustAllowance(consumed_output) => {
                consumed_amount += consumed_output.amount();
            }
            // TODO conflict or error ?
            _ => return Err(Error::UnsupportedOutputType),
        };
    }

    for created_output in transaction.essence().outputs() {
        created_amount += match created_output {
            transaction::Output::SignatureLockedSingle(created_output) => created_output.amount(),
            transaction::Output::SignatureLockedDustAllowance(created_output) => created_output.amount(),
            // TODO conflict or error ?
            _ => return Err(Error::UnsupportedOutputType),
        };
    }

    if consumed_amount != created_amount {
        // TODO conflict or error ?
        return Err(Error::AmountMismatch(consumed_amount, created_amount));
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
    let mut conflict = ConflictReason::None;

    metadata.num_referenced_messages += 1;

    match message.payload() {
        Some(Payload::Transaction(transaction)) => {
            let transaction_id = transaction.id();
            let essence = transaction.essence();
            let mut outputs = HashMap::with_capacity(essence.inputs().len());

            for input in essence.inputs() {
                if let Input::UTXO(utxo_input) = input {
                    let output_id = utxo_input.output_id();

                    // Check if this input was already spent during the confirmation.
                    if metadata.spent_outputs.contains_key(output_id) {
                        conflict = ConflictReason::InputUTXOAlreadySpentInThisMilestone;
                        break;
                    }

                    // Check if this input was newly created during the confirmation.
                    if let Some(output) = metadata.created_outputs.get(output_id).cloned() {
                        outputs.insert(*output_id, output);
                        continue;
                    }

                    // Check current ledger for this input.
                    if let Some(output) = storage::fetch_output(storage.deref(), output_id).await? {
                        // Check if this output is already spent.
                        if !storage::is_output_unspent(storage.deref(), output_id).await? {
                            conflict = ConflictReason::InputUTXOAlreadySpent;
                            break;
                        }
                        outputs.insert(*output_id, output);
                        continue;
                    } else {
                        conflict = ConflictReason::InputUTXONotFound;
                        break;
                    }
                } else {
                    // TODO conflict or error ?
                    return Err(Error::UnsupportedInputType);
                };
            }

            if conflict != ConflictReason::None {
                metadata.excluded_conflicting_messages.push((*message_id, conflict));
                return Ok(());
            }

            match validate_transaction(&transaction, &outputs) {
                Ok(ConflictReason::None) => {
                    // Go through all deposits and generate unspent outputs.
                    for (index, output) in essence.outputs().iter().enumerate() {
                        metadata.created_outputs.insert(
                            // Can't fail because we know the index is valid.
                            OutputId::new(transaction_id, index as u16).unwrap(),
                            Output::new(*message_id, output.clone()),
                        );
                    }
                    for (output_id, _) in outputs {
                        metadata.created_outputs.remove(&output_id);
                        metadata
                            .spent_outputs
                            .insert(output_id, Spent::new(transaction_id, metadata.index));
                    }
                    metadata.included_messages.push(*message_id);
                }
                Ok(conflict) => {
                    metadata.excluded_conflicting_messages.push((*message_id, conflict));
                }
                Err(e) => return Err(e),
            }
        }
        _ => {
            metadata.excluded_no_transaction_messages.push(*message_id);
        }
    };

    Ok(())
}

// TODO make it a tangle method
pub(crate) async fn visit_dfs<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &N::Backend,
    root: MessageId,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: StorageBackend,
{
    let mut messages_ids = vec![root];
    let mut visited = HashSet::new();

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
