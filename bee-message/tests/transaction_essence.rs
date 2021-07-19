// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;
use bee_test::rand::{
    bytes::{rand_bytes, rand_bytes_array},
    number::rand_number,
};

use core::convert::TryInto;

const TRANSACTION_ID: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_1: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const ED25519_ADDRESS_2: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c64f";

#[test]
fn new_valid() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish();

    assert!(essence.is_ok());
}

#[test]
fn invalid_input_count() {
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![])
        .with_outputs(vec![output])
        .finish();

    assert!(matches!(essence.err().unwrap(), ValidationError::InvalidInputCount(0),));
}

#[test]
fn invalid_duplicate_utxo() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let utxo_input = UtxoInput::new(txid, 0).unwrap();
    let input1 = Input::Utxo(utxo_input.clone());
    let input2 = Input::Utxo(utxo_input.clone());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish();

    assert!(matches!(
        essence.err().unwrap(),
        ValidationError::DuplicateUtxo(utxo) if utxo == utxo_input
    ));
}

#[test]
fn invalid_inputs_not_sorted() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish();

    assert!(matches!(
        essence.err().unwrap(),
        ValidationError::TransactionInputsNotSorted,
    ));
}

#[test]
fn invalid_output_count() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![])
        .finish();

    assert!(matches!(essence.err().unwrap(), ValidationError::InvalidOutputCount(0),));
}

#[test]
fn invalid_output_accumulated_amount() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes1: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address1 = Address::from(Ed25519Address::new(bytes1));

    let bytes2: [u8; 32] = hex::decode(ED25519_ADDRESS_2).unwrap().try_into().unwrap();
    let address2 = Address::from(Ed25519Address::new(bytes2));

    let output1 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address1, IOTA_SUPPLY).unwrap());
    let output2 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address2, 1).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output1, output2])
        .finish();

    assert!(matches!(
        essence.err().unwrap(),
        ValidationError::InvalidAccumulatedOutput(amount) if amount == IOTA_SUPPLY as u128 + 1,
    ));
}

#[test]
fn invalid_duplicate_output_address() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));

    let output1 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address.clone(), 1000).unwrap());
    let output2 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address.clone(), 1000).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output1, output2])
        .finish();

    assert!(matches!(
        essence.err().unwrap(),
        ValidationError::DuplicateAddress(duplicate) if duplicate == address,
    ));
}

#[test]
fn invalid_outputs_not_sorted() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes1: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address1 = Address::from(Ed25519Address::new(bytes1));

    let bytes2: [u8; 32] = hex::decode(ED25519_ADDRESS_2).unwrap().try_into().unwrap();
    let address2 = Address::from(Ed25519Address::new(bytes2));

    let output1 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address1, 1000).unwrap());
    let output2 = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address2, 1000).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output2, output1])
        .finish();

    assert!(matches!(
        essence.err().unwrap(),
        ValidationError::TransactionOutputsNotSorted,
    ));
}

#[test]
fn invalid_payload_kind() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let payload = Payload::from(DataPayload::new(0, rand_bytes(128)).unwrap());

    let essence = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .with_payload(payload)
        .finish();

    assert!(matches!(
        essence,
        Err(ValidationError::InvalidPayloadKind(DataPayload::KIND)),
    ));
}

#[test]
fn accessors_eq() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let timestamp = rand_number();
    let access_pledge_id = rand_bytes_array();
    let consensus_pledge_id = rand_bytes_array();
    let inputs = vec![input1.clone(), input2.clone()];
    let outputs = vec![output.clone()];

    let essence = TransactionEssence::builder()
        .with_timestamp(timestamp)
        .with_access_pledge_id(access_pledge_id)
        .with_consensus_pledge_id(consensus_pledge_id)
        .add_input(input1)
        .add_input(input2)
        .add_output(output)
        .finish()
        .unwrap();

    assert_eq!(essence.timestamp(), timestamp);
    assert_eq!(essence.access_pledge_id(), &access_pledge_id);
    assert_eq!(essence.consensus_pledge_id(), &consensus_pledge_id);
    assert_eq!(essence.inputs(), &inputs);
    assert_eq!(essence.outputs(), &outputs);
}

#[test]
fn packed_len() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
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

    assert_eq!(
        essence.packed_len(),
        8 + 32 + 32 + 4 + 2 * (1 + 32 + 2) + 4 + 1 + 1 + 32 + 8 + 1,
    );
}

#[test]
fn round_trip() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());

    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());

    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS_1).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::SignatureLockedSingle(SignatureLockedSingleOutput::new(address, amount).unwrap());

    let essence_a = TransactionEssence::builder()
        .with_timestamp(rand_number())
        .with_access_pledge_id(rand_bytes_array())
        .with_consensus_pledge_id(rand_bytes_array())
        .with_inputs(vec![input1, input2])
        .with_outputs(vec![output])
        .finish()
        .unwrap();

    let essence_b = TransactionEssence::unpack_from_slice(essence_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(essence_a, essence_b);
}
