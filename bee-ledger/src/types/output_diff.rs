// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryDiff};

use bee_common::packable::{Packable as OldPackable, Read, Write};
use bee_message::output::OutputId;
use bee_packable::prefix::{UnpackPrefixError, VecPrefix};

use core::convert::Infallible;

fn unpack_prefix_error_to_error(err: UnpackPrefixError<bee_message::Error, Infallible>) -> Error {
    match err {
        UnpackPrefixError::Packable(err) => err.into(),
        UnpackPrefixError::Prefix(err) => match err {},
    }
}

/// A type to record output and treasury changes that happened within a milestone.
#[derive(Clone, Debug, Eq, PartialEq, bee_packable::Packable)]
#[packable(unpack_error = Error, with = unpack_prefix_error_to_error)]
pub struct OutputDiff {
    created_outputs: VecPrefix<OutputId, u32>,
    consumed_outputs: VecPrefix<OutputId, u32>,
    #[packable(unpack_error_with = |_| Error::PackableOption)]
    treasury_diff: Option<TreasuryDiff>,
}

impl OutputDiff {
    /// Creates a new `OutputDiff`.
    pub fn new(
        created_outputs: Vec<OutputId>,
        consumed_outputs: Vec<OutputId>,
        treasury_diff: Option<TreasuryDiff>,
    ) -> Result<Self, Error> {
        Ok(Self {
            created_outputs: created_outputs.try_into().map_err(Error::InvalidOutputCount)?,
            consumed_outputs: consumed_outputs.try_into().map_err(Error::InvalidOutputCount)?,
            treasury_diff,
        })
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

impl OldPackable for OutputDiff {
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

        self.treasury_diff.pack(writer).map_err(|_| Error::PackableOption)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let created_outputs_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut created_outputs = Vec::with_capacity(created_outputs_len);
        for _ in 0..created_outputs_len {
            created_outputs.push(OutputId::unpack_inner::<R, CHECK>(reader)?);
        }

        let consumed_outputs_len = u32::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut consumed_outputs = Vec::with_capacity(consumed_outputs_len);
        for _ in 0..consumed_outputs_len {
            consumed_outputs.push(OutputId::unpack_inner::<R, CHECK>(reader)?);
        }

        let treasury_diff =
            Option::<TreasuryDiff>::unpack_inner::<R, CHECK>(reader).map_err(|_| Error::PackableOption)?;

        Self::new(created_outputs, consumed_outputs, treasury_diff)
    }
}
