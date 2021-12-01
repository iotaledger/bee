// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{
    address::{Address, Ed25519Address},
    input::{Input, TreasuryInput, UtxoInput},
    milestone::MilestoneIndex,
    output::{Output, SimpleOutput, TreasuryOutput},
    payload::{
        milestone::{MilestoneId, MilestonePayload, MilestonePayloadEssence, MILESTONE_MERKLE_PROOF_LENGTH},
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash},
        transaction::{Essence, RegularEssenceBuilder, TransactionId, TransactionPayloadBuilder},
        IndexationPayload, Payload, TreasuryTransactionPayload,
    },
    signature::{Ed25519Signature, Signature},
    unlock_block::{ReferenceUnlockBlock, SignatureUnlockBlock, UnlockBlock, UnlockBlocks},
};
use bee_test::rand::{bytes::rand_bytes_array, parents::rand_parents};

use std::str::FromStr;

const TRANSACTION_ID: &str = "24a1f46bdb6b2bf38f1c59f73cdd4ae5b418804bb231d76d06fbf246498d5883";
const ED25519_ADDRESS: &str = "e594f9a895c0e0a6760dd12cffc2c3d1e1cbf7269b328091f96ce3d0dd550b75";
const ED25519_PUBLIC_KEY: &str = "1da5ddd11ba3f961acab68fafee3177d039875eaa94ac5fdbff8b53f0c50bfb9";
const ED25519_SIGNATURE: &str = "c6a40edf9a089f42c18f4ebccb35fe4b578d93b879e99b87f63573324a710d3456b03fb6d1fcc027e6401cbd9581f790ee3ed7a3f68e9c225fcb9f1cd7b7110d";
const MESSAGE_ID: &str = "b0212bde21643a8b719f398fe47545c4275b52c1f600e255caa53d77a91bb46d";
const MILESTONE_ID: &str = "40498d437a95fe67c1ed467e6ee85567833c36bf91e71742ea2c71e0633146b9";
const TAIL_TRANSACTION_HASH_BYTES: [u8; 49] = [
    222, 235, 107, 67, 2, 173, 253, 93, 165, 90, 166, 45, 102, 91, 19, 137, 71, 146, 156, 180, 248, 31, 56, 25, 68,
    154, 98, 100, 64, 108, 203, 48, 76, 75, 114, 150, 34, 153, 203, 35, 225, 120, 194, 175, 169, 207, 80, 229, 10,
];

#[test]
fn transaction() {
    let txid = TransactionId::new(hex::decode(TRANSACTION_ID).unwrap().try_into().unwrap());
    let input1 = Input::Utxo(UtxoInput::new(txid, 0).unwrap());
    let input2 = Input::Utxo(UtxoInput::new(txid, 1).unwrap());
    let bytes: [u8; 32] = hex::decode(ED25519_ADDRESS).unwrap().try_into().unwrap();
    let address = Address::from(Ed25519Address::new(bytes));
    let amount = 1_000_000;
    let output = Output::Simple(SimpleOutput::new(address, amount).unwrap());
    let essence = Essence::Regular(
        RegularEssenceBuilder::new()
            .with_inputs(vec![input1, input2])
            .with_outputs(vec![output])
            .finish()
            .unwrap(),
    );

    let pub_key_bytes: [u8; 32] = hex::decode(ED25519_PUBLIC_KEY).unwrap().try_into().unwrap();
    let sig_bytes: [u8; 64] = hex::decode(ED25519_SIGNATURE).unwrap().try_into().unwrap();
    let signature = Ed25519Signature::new(pub_key_bytes, sig_bytes);
    let sig_unlock_block = UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(signature)));
    let ref_unlock_block = UnlockBlock::Reference(ReferenceUnlockBlock::new(0).unwrap());
    let unlock_blocks = UnlockBlocks::new(vec![sig_unlock_block, ref_unlock_block]).unwrap();

    let tx_payload = TransactionPayloadBuilder::new()
        .with_essence(essence)
        .with_unlock_blocks(unlock_blocks)
        .finish()
        .unwrap();

    let payload: Payload = tx_payload.into();
    let packed = payload.pack_new();

    assert_eq!(payload.kind(), 0);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Transaction(_)));
    assert_eq!(payload, Packable::unpack(&mut packed.as_slice()).unwrap());
}

#[test]
fn milestone() {
    let payload: Payload = MilestonePayload::new(
        MilestonePayloadEssence::new(
            MilestoneIndex(0),
            0,
            rand_parents(),
            [0; MILESTONE_MERKLE_PROOF_LENGTH],
            0,
            0,
            vec![[0; 32]],
            None,
        )
        .unwrap(),
        vec![[0; 64]],
    )
    .unwrap()
    .into();

    let packed = payload.pack_new();

    assert_eq!(payload.kind(), 1);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Milestone(_)));
    assert_eq!(payload, Packable::unpack(&mut packed.as_slice()).unwrap());
}

#[test]
fn indexation() {
    let payload: Payload = IndexationPayload::new(&rand_bytes_array::<32>(), &[]).unwrap().into();

    let packed = payload.pack_new();

    assert_eq!(payload.kind(), 2);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Indexation(_)));
}

#[test]
fn receipt() {
    let payload: Payload = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                1_000_000,
            )
            .unwrap(),
        )
        .unwrap()],
        Payload::TreasuryTransaction(Box::new(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(1_000_000).unwrap()),
            )
            .unwrap(),
        )),
    )
    .unwrap()
    .into();

    let packed = payload.pack_new();

    assert_eq!(payload.kind(), 3);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::Receipt(_)));
    assert_eq!(payload, Packable::unpack(&mut packed.as_slice()).unwrap());
}

#[test]
fn treasury_transaction() {
    let payload: Payload = TreasuryTransactionPayload::new(
        Input::from(TreasuryInput::from_str(MESSAGE_ID).unwrap()),
        Output::from(TreasuryOutput::new(1_000_000).unwrap()),
    )
    .unwrap()
    .into();

    let packed = payload.pack_new();

    assert_eq!(payload.kind(), 4);
    assert_eq!(payload.packed_len(), packed.len());
    assert!(matches!(payload, Payload::TreasuryTransaction(_)));
    assert_eq!(payload, Packable::unpack(&mut packed.as_slice()).unwrap());
}
