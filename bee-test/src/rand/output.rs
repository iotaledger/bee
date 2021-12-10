// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    address::rand_address,
    // message::rand_message_id,
    // milestone::{rand_milestone_id, rand_milestone_index},
    number::{rand_number, rand_number_range},
    transaction::rand_transaction_id,
};

// use bee_ledger::types::{ConsumedOutput, CreatedOutput, TreasuryOutput, Unspent};
use bee_message::output::{self, Output, OutputId, SimpleOutput, OUTPUT_INDEX_RANGE};

/// Generates a random output id.
pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_number_range(OUTPUT_INDEX_RANGE)).unwrap()
}

// /// Generates a random unspent output id.
// pub fn rand_unspent_output_id() -> Unspent {
//     Unspent::new(rand_output_id())
// }

/// Generates a random simple output.
pub fn rand_simple_output() -> SimpleOutput {
    SimpleOutput::new(rand_address(), rand_number_range(SimpleOutput::AMOUNT_RANGE)).unwrap()
}

/// Generates a random treasury output.
pub fn rand_treasury_output() -> output::TreasuryOutput {
    output::TreasuryOutput::new(rand_number_range(output::TreasuryOutput::AMOUNT_RANGE)).unwrap()
}

/// Generates a random output.
// TODO finish
pub fn rand_output() -> Output {
    match rand_number::<u64>() % 3 {
        0 => rand_simple_output().into(),
        // 1 => todo!(),
        _ => rand_treasury_output().into(),
        // _ => unreachable!(),
    }
}

// /// Generates a random consumed output.
// pub fn rand_consumed_output() -> ConsumedOutput {
//     ConsumedOutput::new(rand_transaction_id(), rand_milestone_index())
// }
//
// /// Generates a random creates output.
// // TODO finish
// pub fn rand_created_output() -> CreatedOutput {
//     CreatedOutput::new(
//         rand_message_id(),
//         match rand_number::<u64>() % 3 {
//             0 => rand_simple_output().into(),
//             // 1 => todo!(),
//             _ => rand_treasury_output().into(),
//             // _ => unreachable!(),
//         },
//     )
// }
//
// /// Generates a random ledger treasury output.
// pub fn rand_ledger_treasury_output() -> TreasuryOutput {
//     TreasuryOutput::new(rand_treasury_output(), rand_milestone_id())
// }
