// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::rand::{
    block::rand_block_id,
    milestone::{rand_milestone_id, rand_milestone_index},
    number::rand_number,
    output::{rand_output, rand_output_id, rand_treasury_output},
    transaction::rand_transaction_id,
};

use crate::{ConsumedOutput, CreatedOutput, TreasuryOutput, Unspent};

/// Generates a random [`Unspent`] output id.
pub fn rand_unspent_output_id() -> Unspent {
    Unspent::new(rand_output_id())
}

/// Generates a random ledger [`TreasuryOutput`].
pub fn rand_ledger_treasury_output(token_supply: u64) -> TreasuryOutput {
    TreasuryOutput::new(rand_treasury_output(token_supply), rand_milestone_id())
}

/// Generates a random [`ConsumedOutput`].
pub fn rand_consumed_output() -> ConsumedOutput {
    ConsumedOutput::new(rand_transaction_id(), rand_milestone_index(), rand_number())
}

/// Generates a random [`CreatedOutput`].
pub fn rand_created_output(token_supply: u64) -> CreatedOutput {
    CreatedOutput::new(
        rand_block_id(),
        rand_milestone_index(),
        rand_number(),
        rand_output(token_supply),
    )
}
