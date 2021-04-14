// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{input::Input, payload::transaction::Essence, prelude::*};

use core::convert::TryInto;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_PULIC_KEY: &str = "1DA5DDD11BA3F961ACAB68FAFEE3177D039875EAA94AC5FDBFF8B53F0C50BFB9";
const ED25519_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(TransactionPayload::KIND, 0);
}

// Validate that attempting to construct a `TransactionPayload` with no essence is an error.
#[test]
fn builder_no_essence_error() {
    let builder = TransactionPayload::builder();

    assert!(matches!(builder.finish(), Err(Error::MissingField("essence"))));
}

// Validate that attempting to construct a `TransactionPayload` with no unlock blocks is an error.
#[test]
fn builder_no_essence_no_unlock_blocks() {
    // Construct a transaction essence with one input and one output.
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssence::builder()
            .with_inputs(vec![input])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    // Initialize the builder but do not set `with_output_block()`.
    let builder = TransactionPayload::builder().with_essence(essence);

    assert!(matches!(builder.finish(), Err(Error::MissingField("unlock_blocks"))));
}

// Validate that attempting to construct a `TransactionPayload` with too few unlock blocks is an
// error.
#[test]
fn builder_no_essence_too_few_unlock_blocks() {
    // Construct a transaction essence with two inputs and one output.
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssence::builder()
            .with_inputs(vec![input1, input2])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    // Construct a list with a single unlock block, whereas we have 2 tx inputs.
    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PULIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, Box::new(sig_bytes));
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block]).unwrap();

    let builder = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks);

    assert!(matches!(
            builder.finish(),
            Err(Error::InputUnlockBlockCountMismatch(input_len, unlock_blocks_len))
            if input_len == 2 && unlock_blocks_len == 1));
}

// Validate that attempting to construct a `TransactionPayload` with too many unlock blocks is an
// error.
#[test]
fn builder_no_essence_too_many_unlock_blocks() {
    // Construct a transaction essence with one input and one output.
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssence::builder()
            .with_inputs(vec![input1])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    // Construct a list of two unlock blocks, whereas we only have 1 tx input.
    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PULIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, Box::new(sig_bytes));
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());

    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let builder = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks);

    assert!(matches!(
            builder.finish(),
            Err(Error::InputUnlockBlockCountMismatch(input_len, unlock_blocks_len))
            if input_len == 1 && unlock_blocks_len == 2));
}

// Validate that a `unpack` ∘ `pack` round-trip results in the original message.
#[test]
fn pack_unpack_valid() {
    // Construct a transaction essence with two inputs and one output.
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssence::builder()
            .with_inputs(vec![input1, input2])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    // Construct a list of two unlock blocks, whereas we only have 1 tx input.
    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PULIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, Box::new(sig_bytes));
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let tx_payload = TransactionPayload::builder()
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish()
        .unwrap();
    let packed_tx_payload = tx_payload.pack_new();

    assert_eq!(packed_tx_payload.len(), tx_payload.packed_len());
    assert_eq!(tx_payload, Packable::unpack(&mut packed_tx_payload.as_slice()).unwrap());
}
