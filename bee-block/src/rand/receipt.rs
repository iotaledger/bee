// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ternary::{T5B1Buf, Tryte, TryteBuf};
use bytemuck::cast_slice;

use crate::{
    payload::milestone::option::{MigratedFundsEntry, TailTransactionHash},
    rand::{address::rand_address, number::rand_number_range, string::rand_string_charset},
};

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
pub fn rand_migrated_funds_entry(token_supply: u64) -> MigratedFundsEntry {
    MigratedFundsEntry::new(
        rand_tail_transaction_hash(),
        rand_address(),
        rand_number_range(MigratedFundsEntry::AMOUNT_MIN..token_supply),
        token_supply,
    )
    .unwrap()
}
