// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the treasury payload.

use crate::{input::Input, output::Output, Error};

use bee_packable::Packable;

/// `TreasuryTransaction` represents a transaction which moves funds from the treasury.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryTransactionPayload {
    input: Input,
    output: Output,
}

impl TreasuryTransactionPayload {
    /// The payload kind of a `TreasuryTransaction`.
    pub const KIND: u32 = 4;

    /// Creates a new `TreasuryTransaction`.
    pub fn new(input: Input, output: Output) -> Result<Self, Error> {
        if !matches!(input, Input::Treasury(_)) {
            return Err(Error::InvalidInputKind(input.kind()));
        }

        if !matches!(output, Output::Treasury(_)) {
            return Err(Error::InvalidOutputKind(output.kind()));
        }

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
