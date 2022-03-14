// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the treasury payload.

use crate::{input::Input, output::Output, Error};

/// `TreasuryTransaction` represents a transaction which moves funds from the treasury.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryTransactionPayload {
    #[packable(verify_with = verify_input)]
    input: Input,
    #[packable(verify_with = verify_output)]
    output: Output,
}

impl TreasuryTransactionPayload {
    /// The payload kind of a `TreasuryTransaction`.
    pub const KIND: u32 = 4;

    /// Creates a new `TreasuryTransaction`.
    pub fn new(input: Input, output: Output) -> Result<Self, Error> {
        verify_input::<true>(&input)?;
        verify_output::<true>(&output)?;

        Ok(Self { input, output })
    }

    /// Returns the input of a `TreasuryTransaction`.
    pub fn input(&self) -> &Input {
        &self.input
    }

    /// Returns the output of a `TreasuryTransaction`.
    pub fn output(&self) -> &Output {
        &self.output
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
