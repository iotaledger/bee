// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address, bool::rand_bool, integer::rand_integer_range, message::rand_message_id,
    milestone::rand_milestone_index, string::rand_string_charset,
};

use bee_ledger::types::Receipt;
use bee_message::{
    input::{Input, TreasuryInput},
    output::{Output, SignatureLockedSingleOutput, TreasuryOutput, TREASURY_OUTPUT_AMOUNT},
    payload::{
        receipt::{MigratedFundsEntry, ReceiptPayload, TailTransactionHash, MIGRATED_FUNDS_ENTRY_AMOUNT},
        treasury::TreasuryTransactionPayload,
        Payload,
    },
};
use bee_ternary::{T5B1Buf, Tryte, TryteBuf};

use bytemuck::cast_slice;

use std::convert::{TryFrom, TryInto};

// TODO move
pub fn rand_treasury_input() -> Input {
    TreasuryInput::new(rand_message_id()).into()
}

// TODO move
pub fn rand_treasury_output() -> Output {
    TreasuryOutput::new(rand_integer_range(TREASURY_OUTPUT_AMOUNT))
        .unwrap()
        .into()
}

// TODO move
pub fn rand_treasury_transaction() -> Payload {
    TreasuryTransactionPayload::new(rand_treasury_input(), rand_treasury_output())
        .unwrap()
        .into()
}

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

pub fn rand_migrated_funds_entry() -> MigratedFundsEntry {
    MigratedFundsEntry::new(
        rand_tail_transaction_hash(),
        SignatureLockedSingleOutput::new(rand_address(), rand_integer_range(MIGRATED_FUNDS_ENTRY_AMOUNT)).unwrap(),
    )
    .unwrap()
}

pub fn rand_receipt() -> Receipt {
    // TODO rand vector
    Receipt::new(
        ReceiptPayload::new(
            rand_milestone_index(),
            rand_bool(),
            vec![rand_migrated_funds_entry()],
            rand_treasury_transaction(),
        )
        .unwrap(),
        rand_milestone_index(),
    )
}
