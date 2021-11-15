// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::OutputId;

use bee_packable::Packable;

/// An [`Input`](crate::input::Input) referencing an [`Output`](crate::output::Output).
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Packable, Debug)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct UtxoInput(OutputId);

impl UtxoInput {
    /// The [`Input`](crate::input::Input) kind of a [`UtxoInput`].
    pub const KIND: u8 = 0;

    /// Creates a new [`UtxoInput`].
    pub fn new(output_id: OutputId) -> Self {
        Self(output_id)
    }

    /// Returns the [`OutputId`] of a [`UtxoInput`].
    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

impl From<OutputId> for UtxoInput {
    fn from(id: OutputId) -> Self {
        UtxoInput(id)
    }
}

impl core::ops::Deref for UtxoInput {
    type Target = OutputId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
