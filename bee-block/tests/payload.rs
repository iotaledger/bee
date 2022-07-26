// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use bee_block::{
    address::{Address, Ed25519Address},
    input::{Input, TreasuryInput, UtxoInput},
    output::{unlock_condition::AddressUnlockCondition, BasicOutput, Output, TreasuryOutput},
    payload::{
        milestone::{option::MilestoneOptions, MilestoneEssence, MilestoneIndex, MilestonePayload},
        transaction::{RegularTransactionEssence, TransactionEssence, TransactionId, TransactionPayload},
        Payload, TaggedDataPayload, TreasuryTransactionPayload,
    },
    rand::{
        bytes::rand_bytes,
        milestone::{rand_merkle_root, rand_milestone_id},
        output::rand_inputs_commitment,
        parents::rand_parents,
    },
    signature::{Ed25519Signature, Signature},
    unlock::{ReferenceUnlock, SignatureUnlock, Unlock, Unlocks},
};
use packable::PackableExt;

const TRANSACTION_ID: &str = "0x24a1f46bdb6b2bf38f1c59f73cdd4ae5b418804bb231d76d06fbf246498d5883";
const ED25519_ADDRESS: &str = "0xe594f9a895c0e0a6760dd12cffc2c3d1e1cbf7269b328091f96ce3d0dd550b75";
const ED25519_PUBLIC_KEY: &str = "0x1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
const ED25519_SIGNATURE: &str = "0xc6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";
const BLOCK_ID: &str = "0xb0212bde21643a8b719f398fe47545c4275b52c1f600e255caa53d77a91bb46d";

#[test]
fn transaction() {
    let txid = TransactionId::new(prefix_hex::decode(TRANSACTION_ID).unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = prefix_hex::decode(ED25519_ADDRESS).unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Basic(
        BasicOutput::build_with_amount(amount)
            .unwrap()
            .add_unlock_condition(AddressUnlockCondition::new(address).into())
            .finish()
            .unwrap(),
    );
    let essence = TransactionEssence::Regular(
        RegularTransactionEssence::builder(0, rand_inputs_commitment())
            .with_inputs(vec![input1, input2])
            .add_output(output)
            .finish()
            .unwrap(),
    );

    let pub_key_bytes: [u8; 32] = prefix_hex::decode(ED25519_PUBLIC_KEY).unwrap();
    let sig_bytes: [u8; 64] = prefix_hex::decode(ED25519_SIGNATURE).unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock = Unlock::Signature(SignatureUnlock::from(Signature::Ed25519(signature)));
    let ref_unlock = Unlock::Reference(ReferenceUnlock::new(0).unwrap());
    let unlocks = Unlocks::new(vec![sig_unlock, ref_unlock]).unwrap();

    let tx_payload = TransactionPayload::new(essence, unlocks).unwrap();

    let payload: Payload = tx_payload.into();
    let packed = payload.pack_to_vec();

    assert_eq!(payload.kind(), 6);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Transaction(_)));
    assert_eq!(
        payload,
        PackableExt::unpack_verified(&mut packed.as_slice(), &mut ()).unwrap()
    );
}

#[test]
fn milestone() {
    let payload: Payload = MilestonePayload::new(
        MilestoneEssence::new(
            MilestoneIndex(0),
            0,
            rand_milestone_id(),
            rand_parents(),
            rand_merkle_root(),
            rand_merkle_root(),
            vec![],
            MilestoneOptions::new(vec![]).unwrap(),
        )
        .unwrap(),
        vec![Signature::from(Ed25519Signature::new([0; 32], [0; 64]))],
    )
    .unwrap()
    .into();

    let packed = payload.pack_to_vec();

    assert_eq!(payload.kind(), 7);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Milestone(_)));
    assert_eq!(
        payload,
        PackableExt::unpack_verified(&mut packed.as_slice(), &mut ()).unwrap()
    );
}

#[test]
fn tagged_data() {
    let payload: Payload = TaggedDataPayload::new(rand_bytes(32), vec![]).unwrap().into();

    let packed = payload.pack_to_vec();

    assert_eq!(payload.kind(), 5);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::TaggedData(_)));
}

#[test]
fn treasury_transaction() {
    let payload: Payload = TreasuryTransactionPayload::new(
        TreasuryInput::from_str(BLOCK_ID).unwrap(),
        TreasuryOutput::new(1_000_000).unwrap(),
    )
    .unwrap()
    .into();

    let packed = payload.pack_to_vec();

    assert_eq!(payload.kind(), 4);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::TreasuryTransaction(_)));
    assert_eq!(
        payload,
        PackableExt::unpack_verified(&mut packed.as_slice(), &mut ()).unwrap()
    );
}
