// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    /* milestone::rand_milestone_index, */ number::rand_number_range, /* payload::rand_receipt_payload, */
    string::rand_string_charset,
};

// use bee_ledger::types::Receipt;
use bee_message::{
    output::SimpleOutput,
    payload::receipt::{MigratedFundsEntry, TailTransactionHash},
};
use bee_ternary::{T5B1Buf, Tryte, TryteBuf};

use bytemuck::cast_slice;

/// Generates a random tail transaction hash.
pub fn rand_tail_transaction_hash() -> TailTransactionHash {
    let bytes = rand_string_charset("ABCDEFGHIJKLMNOPQRSTUVWXYZ9", 81)
        .chars()
        .map(Tryte::try_from)
        .collect::<Result<TryteBuf, _>>()
        .unwrap()
        .as_trits()
        .encode::<T5B1Buf>();

    TailTransactionHash::new(cast_slice(bytes.as_slice().as_i8_slice()).try_into().unwrap()).unwrap()
}

/// Generates a random migrated funds entry.
pub fn rand_migrated_funds_entry() -> MigratedFundsEntry {
    MigratedFundsEntry::new(
        rand_tail_transaction_hash(),
        SimpleOutput::new(rand_address(), rand_number_range(MigratedFundsEntry::AMOUNT_RANGE)).unwrap(),
    )
    .unwrap()
}

// /// Generates a random ledger receipt.
// pub fn rand_ledger_receipt() -> Receipt {
//     Receipt::new(rand_receipt_payload(), rand_milestone_index())
// }
