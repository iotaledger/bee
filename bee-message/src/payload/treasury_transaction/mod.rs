// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the treasury payload.

use crate::{
    input::{Input, TreasuryInput},
    output::{Output, TreasuryOutput},
    Error,
};

/// [`TreasuryTransaction`] represents a transaction which moves funds from the treasury.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryTransactionPayload {
    #[packable(verify_with = verify_input)]
    input: Input,
    #[packable(verify_with = verify_output)]
    output: Output,
}

impl TreasuryTransactionPayload {
    /// The payload kind of a [`TreasuryTransaction`].
    pub const KIND: u32 = 4;

    /// Creates a new [`TreasuryTransaction`].
    pub fn new(input: TreasuryInput, output: TreasuryOutput) -> Result<Self, Error> {
        Ok(Self {
            input: input.into(),
            output: output.into(),
        })
    }

    /// Returns the input of a [`TreasuryTransaction`].
    pub fn input(&self) -> &TreasuryInput {
        if let Input::Treasury(ref input) = self.input {
            input
        } else {
            unreachable!()
        }
    }

    /// Returns the output of a [`TreasuryTransaction`].
    pub fn output(&self) -> &TreasuryOutput {
        if let Output::Treasury(ref output) = self.output {
            output
        } else {
            unreachable!()
        }
    }
}

fn verify_input<const VERIFY: bool>(input: &Input) -> Result<(), Error> {
    if VERIFY && !matches!(input, Input::Treasury(_)) {
        Err(Error::InvalidInputKind(input.kind()))
    } else {
        Ok(())
    }
}

fn verify_output<const VERIFY: bool>(output: &Output) -> Result<(), Error> {
    if VERIFY && !matches!(output, Output::Treasury(_)) {
        Err(Error::InvalidOutputKind(output.kind()))
    } else {
        Ok(())
    }
}
