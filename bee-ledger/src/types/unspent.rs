// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_message::output::OutputId;
use bee_packable::Packable;

use std::ops::Deref;

/// Represents an output id as unspent.
#[derive(Clone, Eq, PartialEq, Hash, Packable)]
#[packable(unpack_error = Error)]
pub struct Unspent(OutputId);

impl From<OutputId> for Unspent {
    fn from(id: OutputId) -> Self {
        Unspent(id)
    }
}

impl Deref for Unspent {
    type Target = OutputId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Unspent {
    /// Creates a new `Unspent`.
    pub fn new(output_id: OutputId) -> Self {
        output_id.into()
    }

    /// Returns the identifier of an `Unspent`.
    pub fn id(&self) -> &OutputId {
        &self.0
    }
}

impl core::fmt::Display for Unspent {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", *self)
    }
}

impl core::fmt::Debug for Unspent {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Unspent({})", self)
    }
}
