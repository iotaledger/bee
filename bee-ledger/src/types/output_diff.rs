// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::convert::Infallible;

use bee_block::output::OutputId;
use packable::prefix::{UnpackPrefixError, VecPrefix};

use crate::types::{error::Error, TreasuryDiff};

fn unpack_prefix_error_to_error(err: UnpackPrefixError<bee_block::Error, Infallible>) -> Error {
    err.into_item_err().into()
}

/// A type to record output and treasury changes that happened within a milestone.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[packable(unpack_error = Error, with = unpack_prefix_error_to_error)]
pub struct OutputDiff {
    created_outputs: VecPrefix<OutputId, u32>,
    consumed_outputs: VecPrefix<OutputId, u32>,
    #[packable(unpack_error_with = |_| Error::PackableOption)]
    treasury_diff: Option<TreasuryDiff>,
}

impl OutputDiff {
    /// Creates a new [`OutputDiff`].
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

    /// Returns the created outputs of the [`OutputDiff`].
    pub fn created_outputs(&self) -> &[OutputId] {
        &self.created_outputs
    }

    /// Returns the consumed outputs of the [`OutputDiff`].
    pub fn consumed_outputs(&self) -> &[OutputId] {
        &self.consumed_outputs
    }

    /// Returns the treasury diff of the [`OutputDiff`].
    pub fn treasury_diff(&self) -> Option<&TreasuryDiff> {
        self.treasury_diff.as_ref()
    }
}
