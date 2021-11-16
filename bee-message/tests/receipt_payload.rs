// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_test::rand::{bytes::rand_bytes_array, number::rand_number};

use std::str::FromStr;

const AMOUNT: u64 = 1_000_000;
const ED25519_ADDRESS: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MILESTONE_ID: &str = "40498d437a95fe67c1ed467e6ee85567833c36bf91e71742ea2c71e0633146b9";
const TAIL_TRANSACTION_HASH_BYTES: [u8; 49] = [
    222, 235, 107, 67, 2, 173, 253, 93, 165, 90, 166, 45, 102, 91, 19, 137, 71, 146, 156, 180, 248, 31, 56, 25, 68,
    154, 98, 100, 64, 108, 203, 48, 76, 75, 114, 150, 34, 153, 203, 35, 225, 120, 194, 175, 169, 207, 80, 229, 10,
];

#[test]
fn kind() {
    assert_eq!(ReceiptPayload::KIND, 3);
}

#[test]
fn new_valid() {
    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
            )
            .unwrap(),
        )
        .unwrap()],
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    );

    assert!(receipt.is_ok());
}

#[test]
fn new_invalid_receipt_funds_count_low() {
    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![],
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    );

    assert!(matches!(receipt, Err(Error::InvalidReceiptFundsCount(0))));
}

#[test]
fn new_invalid_receipt_funds_count_high() {
    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        false,
        (0..128)
            .map(|_| {
                MigratedFundsEntry::new(
                    TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
                    SimpleOutput::new(
                        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                        AMOUNT,
                    )
                    .unwrap(),
                )
                .unwrap()
            })
            .collect(),
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    );

    assert!(matches!(receipt, Err(Error::InvalidReceiptFundsCount(128))));
}

#[test]
fn new_invalid_payload_kind() {
    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
            )
            .unwrap(),
        )
        .unwrap()],
        Payload::from(IndexationPayload::new(&rand_bytes_array::<32>(), &[]).unwrap()),
    );

    assert!(matches!(
        receipt,
        Err(Error::InvalidPayloadKind(IndexationPayload::KIND))
    ));
}

#[test]
fn new_invalid_transaction_outputs_not_sorted() {
    let mut new_tail_transaction_hash = TAIL_TRANSACTION_HASH_BYTES;
    new_tail_transaction_hash[0] = 223;

    let migrated_funds = vec![
        MigratedFundsEntry::new(
            TailTransactionHash::new(new_tail_transaction_hash).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
            )
            .unwrap(),
        )
        .unwrap(),
        MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
            )
            .unwrap(),
        )
        .unwrap(),
    ];

    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        migrated_funds,
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    );

    assert!(matches!(receipt, Err(Error::ReceiptFundsNotUniqueSorted)));
}

#[test]
fn new_invalid_tail_transaction_hashes_not_unique() {
    let migrated_funds = MigratedFundsEntry::new(
        TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
        SimpleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            AMOUNT,
        )
        .unwrap(),
    )
    .unwrap();

    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![migrated_funds; 2],
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    );

    assert!(matches!(receipt, Err(Error::ReceiptFundsNotUniqueSorted)));
}

#[test]
fn pack_unpack_valid() {
    let receipt = ReceiptPayload::new(
        MilestoneIndex::new(0),
        true,
        vec![MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            SimpleOutput::new(
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
            )
            .unwrap(),
        )
        .unwrap()],
        Payload::from(
            TreasuryTransactionPayload::new(
                Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
                Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
            )
            .unwrap(),
        ),
    )
    .unwrap();

    let packed_receipt = receipt.pack_new();

    assert_eq!(packed_receipt.len(), receipt.packed_len());
    assert_eq!(receipt, Packable::unpack(&mut packed_receipt.as_slice()).unwrap());
}

#[test]
fn getters() {
    let migrated_at = MilestoneIndex::new(rand_number());
    let last = true;
    let funds = vec![MigratedFundsEntry::new(
        TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
        SimpleOutput::new(
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            AMOUNT,
        )
        .unwrap(),
    )
    .unwrap()];
    let payload: Payload = TreasuryTransactionPayload::new(
        Input::Treasury(TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap())),
        Output::Treasury(TreasuryOutput::new(AMOUNT).unwrap()),
    )
    .unwrap()
    .into();

    let receipt = ReceiptPayload::new(migrated_at, last, funds.clone(), payload.clone()).unwrap();

    assert_eq!(receipt.migrated_at(), migrated_at);
    assert_eq!(receipt.last(), last);
    assert_eq!(receipt.funds(), funds.as_slice());
    assert_eq!(*receipt.transaction(), payload);
    assert_eq!(receipt.amount(), AMOUNT);
}
