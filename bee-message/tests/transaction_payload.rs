// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    address::{Address, Ed25519Address},
    input::{Input, UtxoInput},
    output::{unlock_condition::AddressUnlockCondition, BasicOutput, Output},
    payload::transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
    signature::{Ed25519Signature, Signature},
    unlock_block::{ReferenceUnlockBlock, SignatureUnlockBlock, UnlockBlock, UnlockBlocks},
    Error,
};
use bee_test::rand::bytes::rand_bytes_array;
use packable::PackableExt;

const TRANSACTION_ID: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_PUBLIC_KEY: &str = "0x1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
const ED25519_SIGNATURE: &str = "0xc6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(TransactionPayload::KIND, 6);
}

// Validate that attempting to construct a `TransactionPayload` with too few unlock blocks is an
// error.
#[test]
fn builder_no_essence_too_few_unlock_blocks() {
    // Construct a transaction essence with two inputs and one output.
    let txid = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(0, rand_bytes_array())
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish()
            .unwrap(),
    );

    // Construct a list with a single unlock block, whereas we have 2 tx inputs.
    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(signature)));
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block]).unwrap();

    assert!(matches!(
            TransactionPayload::new(essence, unlock_blocks),
            Err(Error::InputUnlockBlockCountMismatch{input_count, block_count})
            if input_count == 2 && block_count == 1));
}

// Validate that attempting to construct a `TransactionPayload` with too many unlock blocks is an
// error.
#[test]
fn builder_no_essence_too_many_unlock_blocks() {
    // Construct a transaction essence with one input and one output.
    let txid = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(0, rand_bytes_array())
            .add_input(input1)
            .add_output(output)
            .finish()
            .unwrap(),
    );

    // Construct a list of two unlock blocks, whereas we only have 1 tx input.
    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(signature)));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlockBlock::new(0).unwrap());

    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    assert!(matches!(
            TransactionPayload::new(essence, unlock_blocks),
            Err(Error::InputUnlockBlockCountMismatch{input_count, block_count})
            if input_count == 1 && block_count == 2));
}

// Validate that a `unpack` ∘ `pack` round-trip results in the original message.
#[test]
fn pack_unpack_valid() {
    // Construct a transaction essence with two inputs and one output.
    let txid = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(0, rand_bytes_array())
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish()
            .unwrap(),
    );

    // Construct a list of two unlock blocks, whereas we only have 1 tx input.
    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(signature)));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlockBlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let tx_payload = TransactionPayload::new(essence, unlock_blocks).unwrap();
    let packed_tx_payload = tx_payload.pack_to_vec();

    assert_eq!(packed_tx_payload.len(), tx_payload.packed_len());
    assert_eq!(
        tx_payload,
        PackableExt::unpack_verified(&mut packed_tx_payload.as_slice()).unwrap()
    );
}

#[test]
fn getters() {
    // Construct a transaction essence with two inputs and one output.
    let txid = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(0, rand_bytes_array())
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish()
            .unwrap(),
    );

    // Construct a list of two unlock blocks, whereas we only have 1 tx input.
    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(signature)));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlockBlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let tx_payload = TransactionPayload::new(essence.clone(), unlock_blocks.clone()).unwrap();

    assert_eq!(*tx_payload.essence(), essence);
    assert_eq!(*tx_payload.unlock_blocks(), unlock_blocks);
}
