// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::{treasury_diff::TreasuryDiff, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_message::output::OutputId;

#[derive(Debug)]
pub struct OutputDiff {
    created_outputs: Vec<OutputId>,
    consumed_outputs: Vec<OutputId>,
    treasury_diff: Option<TreasuryDiff>,
}

impl OutputDiff {
    pub fn new(
        created_outputs: Vec<OutputId>,
        consumed_outputs: Vec<OutputId>,
        treasury_diff: Option<TreasuryDiff>,
    ) -> Self {
        Self {
            created_outputs,
            consumed_outputs,
            treasury_diff,
        }
    }

    pub fn created_outputs(&self) -> &[OutputId] {
        &self.created_outputs
    }

    pub fn consumed_outputs(&self) -> &[OutputId] {
        &self.consumed_outputs
    }

    pub fn treasury_diff(&self) -> Option<&TreasuryDiff> {
        self.treasury_diff.as_ref()
    }
}

impl Packable for OutputDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.created_outputs.packed_len() + self.consumed_outputs.packed_len() + self.treasury_diff.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.created_outputs.len() as u32).pack(writer)?;
        for output in self.created_outputs.iter() {
            output.pack(writer)?;
        }

        (self.consumed_outputs.len() as u32).pack(writer)?;
        for output in self.consumed_outputs.iter() {
            output.pack(writer)?;
        }

        self.treasury_diff.pack(writer).map_err(|_| Error::Option)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let created_outputs_len = u32::unpack(reader)? as usize;
        let mut created_outputs = Vec::with_capacity(created_outputs_len);
        for _ in 0..created_outputs_len {
            created_outputs.push(OutputId::unpack(reader)?);
        }

        let consumed_outputs_len = u32::unpack(reader)? as usize;
        let mut consumed_outputs = Vec::with_capacity(consumed_outputs_len);
        for _ in 0..consumed_outputs_len {
            consumed_outputs.push(OutputId::unpack(reader)?);
        }

        Ok(Self {
            created_outputs,
            consumed_outputs,
            treasury_diff: Option::<TreasuryDiff>::unpack(reader).map_err(|_| Error::Option)?,
        })
    }
}
