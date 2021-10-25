// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryDiff};

use bee_message::output::OutputId;
use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

/// A type to record output and treasury changes that happened within a milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputDiff {
    created_outputs: Vec<OutputId>,
    consumed_outputs: Vec<OutputId>,
    treasury_diff: Option<TreasuryDiff>,
}

impl OutputDiff {
    /// Creates a new `OutputDiff`.
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

    /// Returns the created outputs of the `OutputDiff`.
    pub fn created_outputs(&self) -> &[OutputId] {
        &self.created_outputs
    }

    /// Returns the consumed outputs of the `OutputDiff`.
    pub fn consumed_outputs(&self) -> &[OutputId] {
        &self.consumed_outputs
    }

    /// Returns the treasury diff of the `OutputDiff`.
    pub fn treasury_diff(&self) -> Option<&TreasuryDiff> {
        self.treasury_diff.as_ref()
    }
}

impl Packable for OutputDiff {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (self.created_outputs.len() as u32).pack(packer)?;
        for output in self.created_outputs.iter() {
            output.pack(packer)?;
        }

        (self.consumed_outputs.len() as u32).pack(packer)?;
        for output in self.consumed_outputs.iter() {
            output.pack(packer)?;
        }

        self.treasury_diff.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let created_outputs_len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let mut created_outputs = Vec::with_capacity(created_outputs_len);
        for _ in 0..created_outputs_len {
            created_outputs.push(OutputId::unpack::<_, VERIFY>(unpacker).coerce()?);
        }

        let consumed_outputs_len = u32::unpack::<_, VERIFY>(unpacker).infallible()? as usize;
        let mut consumed_outputs = Vec::with_capacity(consumed_outputs_len);
        for _ in 0..consumed_outputs_len {
            consumed_outputs.push(OutputId::unpack::<_, VERIFY>(unpacker).coerce()?);
        }
        let treasury_diff =
            Option::<TreasuryDiff>::unpack::<_, VERIFY>(unpacker).map_packable_err(|_| Error::PackableOption)?;

        Ok(Self {
            created_outputs,
            consumed_outputs,
            treasury_diff,
        })
    }
}
