// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the FPC statement payload.

mod conflicts;
mod timestamps;

pub use conflicts::{Conflict, Conflicts};
pub use timestamps::{Timestamp, Timestamps};

use crate::{payload::MessagePayload, MessageUnpackError, ValidationError};

use bee_packable::Packable;

/// Payload describing opinions on conflicts and timestamps of messages.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct FpcPayload {
    /// Collection of opinions on conflicting transactions.
    conflicts: Conflicts,
    /// Collection of opinions on message timestamps.
    timestamps: Timestamps,
}

impl MessagePayload for FpcPayload {
    const KIND: u32 = 2;
    const VERSION: u8 = 0;
}

impl FpcPayload {
    /// Returns a new [`FpcPayloadBuilder`] in order to build an [`FpcPayload`].
    pub fn builder() -> FpcPayloadBuilder {
        FpcPayloadBuilder::new()
    }

    /// Returns the [`Conflicts`] of an [`FpcPayload`].
    pub fn conflicts(&self) -> impl Iterator<Item = &Conflict> {
        self.conflicts.iter()
    }

    /// Returns the [`Timestamps`] of an [`FpcPayload`].
    pub fn timestamps(&self) -> impl Iterator<Item = &Timestamp> {
        self.timestamps.iter()
    }
}

/// A builder to build an [`FpcPayload`].
#[derive(Default)]
pub struct FpcPayloadBuilder {
    conflicts: Conflicts,
    timestamps: Timestamps,
}

impl FpcPayloadBuilder {
    /// Creates a new [`FpcPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a collection of conflicts to the [`FpcPayloadBuilder`].
    pub fn with_conflicts(mut self, conflicts: Conflicts) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Adds a collection of timestamps to the [`FpcPayloadBuilder`].
    pub fn with_timestamps(mut self, timestamps: Timestamps) -> Self {
        self.timestamps = timestamps;
        self
    }

    /// Finishes an [`FpcPayloadBuilder`] into an [`FpcPayload`].
    pub fn finish(self) -> Result<FpcPayload, ValidationError> {
        Ok(FpcPayload {
            conflicts: self.conflicts,
            timestamps: self.timestamps,
        })
    }
}
