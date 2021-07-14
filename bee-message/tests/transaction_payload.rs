// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;
use bee_test::rand::{bytes::rand_bytes_array, number::rand_number};

use core::convert::TryInto;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_PUBLIC_KEY: &str = "1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
const ED25519_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401\
    cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";

#[test]
fn kind() {
    assert_eq!(TransactionPayload::KIND, 0);
}

#[test]
fn invalid_no_essence() {
    let payload = TransactionPayload::builder().with_version(0).finish();

    assert!(matches!(payload.unwrap_err(), ValidationError::MissingField("essence"),));
}

#[test]
fn invalid_no_unlock_blocks() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let payload = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence)
        .finish();

    assert!(matches!(
        payload.unwrap_err(),
        ValidationError::MissingField("unlock_blocks"),
    ));
}

#[test]
fn invalid_too_few_unlock_blocks() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block]).unwrap();

    let payload = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish();

    assert!(matches!(
        payload.unwrap_err(),
        ValidationError::InputUnlockBlockCountMismatch(input_len, unlock_blocks_len)
            if input_len == 2 && unlock_blocks_len == 1,
    ));
}

#[test]
fn invalid_too_many_unlock_blocks() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input = Input::Utxo(UtxoInput::new(txid, 0).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let payload = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish();

    assert!(matches!(
        payload.unwrap_err(),
        ValidationError::InputUnlockBlockCountMismatch(input_len, unlock_blocks_len)
            if input_len == 1 && unlock_blocks_len == 2,
    ));
}

#[test]
fn accessors_eq() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let payload = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence.clone())
        .with_unlock_blocks(unlock_blocks.clone())
        .finish()
        .unwrap();

    assert_eq!(payload.version(), 0);
    assert_eq!(*payload.essence(), essence);
    assert_eq!(*payload.unlock_blocks(), unlock_blocks);
}

#[test]
fn packed_len() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let payload = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish()
        .unwrap();

    assert_eq!(
        payload.packed_len(),
        1 + 8 + 32 + 32 + 4 + 2 * (1 + 32 + 2) + 4 + 1 + 1 + 32 + 8 + 1 + 2 + 1 + 1 + 32 + 64 + 1 + 2,
    )
}

#[test]
fn round_trip() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlock::Ed25519(signature));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let payload_a = TransactionPayload::builder()
        .with_version(0)
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish()
        .unwrap();

    let payload_b = TransactionPayload::unpack_from_slice(payload_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(payload_a, payload_b);
    assert_eq!(payload_a.id(), payload_b.id());
}
