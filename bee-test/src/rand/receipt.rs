// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{address::rand_address, integer::rand_integer_range, string::rand_string_charset};

use bee_message::{
    output::SignatureLockedSingleOutput,
    payload::receipt::{MigratedFundsEntry, TailTransactionHash, MIGRATED_FUNDS_ENTRY_AMOUNT},
};
use bee_ternary::{T5B1Buf, Tryte, TryteBuf};

use bytemuck::cast_slice;

use std::convert::{TryFrom, TryInto};

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
        SignatureLockedSingleOutput::new(rand_address(), rand_integer_range(MIGRATED_FUNDS_ENTRY_AMOUNT)).unwrap(),
    )
    .unwrap()
}
