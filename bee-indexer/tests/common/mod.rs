// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_ledger::{types::CreatedOutput, workers::event::OutputCreated};
use bee_message::output::Output;
use bee_test::rand::{
    message::rand_message_id,
    milestone::rand_milestone_index,
    number::rand_number,
    output::{rand_alias_output, rand_output_id},
};

pub(crate) fn rand_output_created_alias() -> OutputCreated {
    OutputCreated {
        output_id: rand_output_id(),
        output: CreatedOutput::new(
            rand_message_id(),
            rand_milestone_index(),
            rand_number(),
            Output::Alias(rand_alias_output()),
        ),
    }
}
