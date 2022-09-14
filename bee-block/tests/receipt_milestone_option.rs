// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use bee_block::{
    address::{Address, Ed25519Address},
    input::TreasuryInput,
    output::TreasuryOutput,
    payload::{
        milestone::{
            option::{MigratedFundsEntry, ReceiptMilestoneOption, TailTransactionHash},
            MilestoneId, MilestoneIndex,
        },
        TreasuryTransactionPayload,
    },
    protocol::protocol_parameters,
    rand::number::rand_number,
    Error,
};
use packable::{bounded::TryIntoBoundedU16Error, PackableExt};

const AMOUNT: u64 = 1_000_000;
const ED25519_ADDRESS: &str = "0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";
const MILESTONE_ID: &str = "0x40498d437a95fe67c1ed467e6ee85567833c36bf91e71742ea2c71e0633146b9";
const TAIL_TRANSACTION_HASH_BYTES: [u8; 49] = [
    222, 235, 107, 67, 2, 173, 253, 93, 165, 90, 166, 45, 102, 91, 19, 137, 71, 146, 156, 180, 248, 31, 56, 25, 68,
    154, 98, 100, 64, 108, 203, 48, 76, 75, 114, 150, 34, 153, 203, 35, 225, 120, 194, 175, 169, 207, 80, 229, 10,
];

#[test]
fn kind() {
    assert_eq!(ReceiptMilestoneOption::KIND, 0);
}

#[test]
fn new_valid() {
    let protocol_parameters = protocol_parameters();
    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        true,
        vec![
            MigratedFundsEntry::new(
                TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
                &protocol_parameters,
            )
            .unwrap(),
        ],
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    );

    assert!(receipt.is_ok());
}

#[test]
fn new_invalid_receipt_funds_count_low() {
    let protocol_parameters = protocol_parameters();
    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        true,
        vec![],
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    );

    assert!(matches!(
        receipt,
        Err(Error::InvalidReceiptFundsCount(TryIntoBoundedU16Error::Invalid(0)))
    ));
}

#[test]
fn new_invalid_receipt_funds_count_high() {
    let protocol_parameters = protocol_parameters();
    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        false,
        (0..129)
            .map(|_| {
                MigratedFundsEntry::new(
                    TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
                    Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                    AMOUNT,
                    &protocol_parameters,
                )
                .unwrap()
            })
            .collect(),
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    );

    assert!(matches!(
        receipt,
        Err(Error::InvalidReceiptFundsCount(TryIntoBoundedU16Error::Invalid(129)))
    ));
}

#[test]
fn new_invalid_transaction_outputs_not_sorted() {
    let protocol_parameters = protocol_parameters();
    let mut new_tail_transaction_hash = TAIL_TRANSACTION_HASH_BYTES;
    new_tail_transaction_hash[0] = 223;

    let migrated_funds = vec![
        MigratedFundsEntry::new(
            TailTransactionHash::new(new_tail_transaction_hash).unwrap(),
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            AMOUNT,
            &protocol_parameters,
        )
        .unwrap(),
        MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            AMOUNT,
            &protocol_parameters,
        )
        .unwrap(),
    ];

    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        true,
        migrated_funds,
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    );

    assert!(matches!(receipt, Err(Error::ReceiptFundsNotUniqueSorted)));
}

#[test]
fn new_invalid_tail_transaction_hashes_not_unique() {
    let protocol_parameters = protocol_parameters();
    let migrated_funds = MigratedFundsEntry::new(
        TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
        Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
        AMOUNT,
        &protocol_parameters,
    )
    .unwrap();

    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        true,
        vec![migrated_funds; 2],
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    );

    assert!(matches!(receipt, Err(Error::ReceiptFundsNotUniqueSorted)));
}

#[test]
fn pack_unpack_valid() {
    let protocol_parameters = protocol_parameters();
    let receipt = ReceiptMilestoneOption::new(
        MilestoneIndex::new(0),
        true,
        vec![
            MigratedFundsEntry::new(
                TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
                Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
                AMOUNT,
                &protocol_parameters,
            )
            .unwrap(),
        ],
        TreasuryTransactionPayload::new(
            TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
            TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
        )
        .unwrap(),
        protocol_parameters.token_supply(),
    )
    .unwrap();

    let packed_receipt = receipt.pack_to_vec();

    assert_eq!(packed_receipt.len(), receipt.packed_len());
    assert_eq!(
        receipt,
        PackableExt::unpack_verified(&mut packed_receipt.as_slice(), &protocol_parameters).unwrap()
    );
}

#[test]
fn getters() {
    let protocol_parameters = protocol_parameters();
    let migrated_at = MilestoneIndex::new(rand_number());
    let last = true;
    let funds = vec![
        MigratedFundsEntry::new(
            TailTransactionHash::new(TAIL_TRANSACTION_HASH_BYTES).unwrap(),
            Address::from(Ed25519Address::from_str(ED25519_ADDRESS).unwrap()),
            AMOUNT,
            &protocol_parameters,
        )
        .unwrap(),
    ];
    let transaction = TreasuryTransactionPayload::new(
        TreasuryInput::new(MilestoneId::from_str(MILESTONE_ID).unwrap()),
        TreasuryOutput::new(AMOUNT, protocol_parameters.token_supply()).unwrap(),
    )
    .unwrap();

    let receipt = ReceiptMilestoneOption::new(
        migrated_at,
        last,
        funds.clone(),
        transaction.clone(),
        protocol_parameters.token_supply(),
    )
    .unwrap();

    assert_eq!(receipt.migrated_at(), migrated_at);
    assert_eq!(receipt.last(), last);
    assert_eq!(receipt.funds(), funds.as_slice());
    assert_eq!(receipt.transaction(), &transaction);
    assert_eq!(receipt.amount(), AMOUNT);
}
