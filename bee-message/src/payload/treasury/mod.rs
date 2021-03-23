// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{input::Input, output::Output, Error};

use bee_common::packable::{Packable, Read, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TreasuryTransactionPayload {
    input: Input,
    output: Output,
}

impl TreasuryTransactionPayload {
    pub const KIND: u32 = 4;

    pub fn new(input: Input, output: Output) -> Result<Self, Error> {
        if !matches!(input, Input::Treasury(_)) {
            return Err(Error::InvalidInputKind(input.kind()));
        }

        if !matches!(output, Output::Treasury(_)) {
            return Err(Error::InvalidOutputKind(output.kind()));
        }

        Ok(Self { input, output })
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let input = Input::unpack(reader)?;
        let output = Output::unpack(reader)?;

        Self::new(input, output)
    }
}
