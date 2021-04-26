// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    integer::{rand_integer, rand_integer_range},
    message::rand_message_id,
    milestone::rand_milestone_index,
    transaction::rand_transaction_id,
};

use bee_ledger::types::{ConsumedOutput, CreatedOutput, Unspent};
use bee_message::{
    constants::{INPUT_OUTPUT_INDEX_RANGE, IOTA_SUPPLY},
    output::{
        OutputId, SignatureLockedDustAllowanceOutput, SignatureLockedSingleOutput, TreasuryOutput, DUST_THRESHOLD,
        TREASURY_OUTPUT_AMOUNT,
    },
};

pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_integer_range(INPUT_OUTPUT_INDEX_RANGE)).unwrap()
}

pub fn rand_unspent_output_id() -> Unspent {
    Unspent::new(rand_output_id())
}

pub fn rand_signature_locked_single_output() -> SignatureLockedSingleOutput {
    // TODO replace with contant range from bee-message
    SignatureLockedSingleOutput::new(rand_address(), rand_integer_range(1..IOTA_SUPPLY)).unwrap()
}

pub fn rand_signature_locked_dust_allowance_output() -> SignatureLockedDustAllowanceOutput {
    // TODO replace with contant range from bee-message
    SignatureLockedDustAllowanceOutput::new(rand_address(), rand_integer_range(DUST_THRESHOLD..IOTA_SUPPLY)).unwrap()
}

pub fn rand_treasury_output() -> TreasuryOutput {
    TreasuryOutput::new(rand_integer_range(TREASURY_OUTPUT_AMOUNT)).unwrap()
}

pub fn rand_consumed_output() -> ConsumedOutput {
    ConsumedOutput::new(rand_transaction_id(), rand_milestone_index())
}

pub fn rand_created_output() -> CreatedOutput {
    CreatedOutput::new(
        rand_message_id(),
        match rand_integer::<u64>() % 3 {
            0 => rand_signature_locked_single_output().into(),
            1 => rand_signature_locked_dust_allowance_output().into(),
            2 => rand_treasury_output().into(),
            _ => unimplemented!(),
        },
    )
}
