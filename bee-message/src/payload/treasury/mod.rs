// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the treasury payload.

use crate::{input::Input, output::Output, Error};

use bee_common::packable::{Packable, Read, Write};

/// `TreasuryTransaction` represents a transaction which moves funds from the treasury.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

impl Packable for TreasuryTransactionPayload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.input.packed_len() + self.output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.input.pack(writer)?;
        self.output.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let input = Input::unpack_inner::<R, CHECK>(reader)?;
        let output = Output::unpack_inner::<R, CHECK>(reader)?;

        Self::new(input, output)
    }
}
