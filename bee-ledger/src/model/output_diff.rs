// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::{milestone::MilestoneId, transaction::OutputId};

#[derive(Debug)]
pub struct TreasuryDiff {
    created: MilestoneId,
    consumed: MilestoneId,
}

impl TreasuryDiff {
    pub fn new(created: MilestoneId, consumed: MilestoneId) -> Self {
        Self { created, consumed }
    }

    pub fn created(&self) -> &MilestoneId {
        &self.created
    }

    pub fn consumed(&self) -> &MilestoneId {
        &self.consumed
    }
}

impl Packable for TreasuryDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.created.packed_len() + self.consumed.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.created.pack(writer)?;
        self.consumed.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            created: MilestoneId::unpack(reader)?,
            consumed: MilestoneId::unpack(reader)?,
        })
    }
}

#[derive(Debug)]
pub struct OutputDiff {
    created_outputs: Vec<OutputId>,
    consumed_outputs: Vec<OutputId>,
    treasury: Option<TreasuryDiff>,
}

impl OutputDiff {
    pub fn new(
        created_outputs: Vec<OutputId>,
        consumed_outputs: Vec<OutputId>,
        treasury: Option<TreasuryDiff>,
    ) -> Self {
        Self {
            created_outputs,
            consumed_outputs,
            treasury,
        }
    }

    pub fn created_outputs(&self) -> &[OutputId] {
        &self.created_outputs
    }

    pub fn consumed_outputs(&self) -> &[OutputId] {
        &self.consumed_outputs
    }

    pub fn treasury(&self) -> Option<&TreasuryDiff> {
        self.treasury.as_ref()
    }
}

impl Packable for OutputDiff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.created_outputs.packed_len() + self.consumed_outputs.packed_len() + self.treasury.packed_len()
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

        self.treasury.pack(writer).map_err(|_| Error::Option)?;

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
            treasury: Option::<TreasuryDiff>::unpack(reader).map_err(|_| Error::Option)?,
        })
    }
}
