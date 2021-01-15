// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::OutputId;

#[derive(Debug)]
pub struct Diff {
    spent_outputs: Vec<OutputId>,
    created_outputs: Vec<OutputId>,
}

impl Diff {
    pub fn new(spent_outputs: Vec<OutputId>, created_outputs: Vec<OutputId>) -> Self {
        Self {
            spent_outputs,
            created_outputs,
        }
    }

    pub fn spent_outputs(&self) -> &[OutputId] {
        &self.spent_outputs
    }

    pub fn created_outputs(&self) -> &[OutputId] {
        &self.created_outputs
    }
}

impl Packable for Diff {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.spent_outputs.packed_len() + self.created_outputs.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.spent_outputs.len() as u32).pack(writer)?;
        for spent_output in self.spent_outputs.iter() {
            spent_output.pack(writer)?;
        }
        (self.created_outputs.len() as u32).pack(writer)?;
        for created_output in self.created_outputs.iter() {
            created_output.pack(writer)?;
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let spent_outputs_len = u32::unpack(reader)? as usize;
        let mut spent_outputs = Vec::with_capacity(spent_outputs_len);
        for _ in 0..spent_outputs_len {
            spent_outputs.push(OutputId::unpack(reader)?);
        }

        let created_outputs_len = u32::unpack(reader)? as usize;
        let mut created_outputs = Vec::with_capacity(created_outputs_len);
        for _ in 0..created_outputs_len {
            created_outputs.push(OutputId::unpack(reader)?);
        }

        Ok(Self {
            spent_outputs,
            created_outputs,
        })
    }
}
