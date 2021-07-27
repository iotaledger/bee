// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{MessageUnpackError, ValidationError},
    output::OutputId,
    payload::transaction::TransactionId,
};

use bee_packable::packable::Packable;

use core::{convert::From, str::FromStr};

/// Represents an input referencing an output.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct UtxoInput(OutputId);

impl UtxoInput {
    /// The input kind of a [`UtxoInput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`UtxoInput`].
    pub fn new(id: TransactionId, index: u16) -> Result<Self, ValidationError> {
        Ok(Self(OutputId::new(id, index)?))
    }

    /// Returns the output id of a [`UtxoInput`].
    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

impl From<OutputId> for UtxoInput {
    fn from(id: OutputId) -> Self {
        UtxoInput(id)
    }
}

impl FromStr for UtxoInput {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtxoInput(OutputId::from_str(s)?))
    }
}

impl core::fmt::Display for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UtxoInput({})", self.0)
    }
}
