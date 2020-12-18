// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::Error,
    metadata::WhiteFlagMetadata,
    model::{Output, Spent},
    storage::{self, Backend},
};

use bee_common_pt2::node::{Node, ResHandle};
use bee_message::{
    payload::{
        transaction::{Input, OutputId},
        Payload,
    },
    Message, MessageId,
};
use bee_protocol::tangle::MsTangle;

use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

// const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[inline]
async fn on_message<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    message_id: &MessageId,
    message: &Message,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let mut conflicting = false;

    metadata.num_messages_referenced += 1;

    if let Some(Payload::Transaction(transaction)) = message.payload() {
        let transaction_id = transaction.id();
        let essence = transaction.essence();

        let mut outputs = HashMap::with_capacity(essence.inputs().len());

        // TODO check transaction syntax here ?

        for input in essence.inputs() {
            if let Input::UTXO(utxo_input) = input {
                let output_id = utxo_input.output_id();

                // Check if this input was already spent during the confirmation.
                if metadata.spent_outputs.contains_key(output_id) {
                    conflicting = true;
                    break;
                }

                // Check if this input was newly created during the confirmation.
                if let Some(output) = metadata.created_outputs.get(output_id).cloned() {
                    outputs.insert(output_id, output);
                    continue;
                }

                // Check current ledger for this input.
                if let Some(output) = storage::fetch_output(storage.deref(), output_id).await? {
                    // Check if this output is already spent.
                    if !storage::is_output_unspent(storage.deref(), output_id).await? {
                        conflicting = true;
                        break;
                    }
                    outputs.insert(output_id, output);
                } else {
                    // TODO conflicting ?
                    conflicting = true;
                    break;
                }
            } else {
                return Err(Error::UnsupportedInputType);
            };
        }

        // TODO semantic validation
        // Verify that all outputs consume all inputs and have valid signatures. Also verify that the amounts match.

        if conflicting {
            metadata.num_messages_excluded_conflicting += 1;
        } else {
            // Go through all deposits and generate unspent outputs.
            for (index, output) in essence.outputs().iter().enumerate() {
                metadata.created_outputs.insert(
                    // Can't fail because we know the index is valid.
                    OutputId::new(transaction_id, index as u16).unwrap(),
                    Output::new(*message_id, output.clone()),
                );
            }
            for (output_id, _) in outputs {
                metadata.created_outputs.remove(output_id);
                metadata
                    .spent_outputs
                    .insert(*output_id, Spent::new(transaction_id, metadata.index));
            }
            metadata.messages_included.push(*message_id);
        }
    } else {
        metadata.num_messages_excluded_no_transaction += 1;
    }

    tangle.update_metadata(message_id, |message_metadata| {
        message_metadata.flags_mut().set_conflicting(conflicting);
        message_metadata.set_milestone_index(metadata.index);
        // TODO pass actual ms timestamp
        message_metadata.confirm();
    });

    Ok(())
}

// TODO make it a tangle method
pub(crate) async fn visit_dfs<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    root: MessageId,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let mut messages_ids = vec![root];
    let mut visited = HashSet::new();

    // TODO Tangle get message AND meta at the same time

    while let Some(message_id) = messages_ids.last() {
        let meta = match tangle.get_metadata(message_id) {
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
                    on_message::<N>(tangle, storage, message_id, &message, metadata).await?;
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
