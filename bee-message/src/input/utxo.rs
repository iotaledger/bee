// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::OutputId, payload::transaction::TransactionId, Error};

use bee_packable::Packable;

use core::{convert::From, str::FromStr};

/// Represents an input referencing an output.
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
pub struct UtxoInput(OutputId);

impl UtxoInput {
    /// The input kind of a `UtxoInput`.
    pub const KIND: u8 = 0;

    /// Creates a new `UtxoInput`.
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        Ok(Self(OutputId::new(id, index)?))
    }

    /// Returns the output id of a `UtxoInput`.
    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(UtxoInput);

impl From<OutputId> for UtxoInput {
    fn from(id: OutputId) -> Self {
        UtxoInput(id)
    }
}

impl FromStr for UtxoInput {
    type Err = Error;

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
