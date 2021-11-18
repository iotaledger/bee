// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the FPC statement payload.

mod conflict;
mod opinion;
mod timestamp;

pub use conflict::Conflict;
pub use opinion::{Opinion, OpinionUnpackError};
pub use timestamp::Timestamp;

use crate::{
    payload::{transaction::TransactionId, MessagePayload, PAYLOAD_LENGTH_MAX},
    MessageId, MessageUnpackError, ValidationError,
};

use bee_packable::{
    bounded::BoundedU32,
    prefix::{UnpackPrefixError, VecPrefix},
    Packable,
};

use alloc::vec::Vec;

/// No [`Vec`] max length specified, so use [`PAYLOAD_LENGTH_MAX`] / length of [`Conflict`].
pub(crate) type ConflictsCount =
    BoundedU32<0, { PAYLOAD_LENGTH_MAX / (TransactionId::LENGTH + 2 * core::mem::size_of::<u8>()) as u32 }>;
/// No [`Vec`] max length specified, so use [`PAYLOAD_LENGTH_MAX`] / length of [`Timestamp`].
pub(crate) type TimestampsCount =
    BoundedU32<0, { PAYLOAD_LENGTH_MAX / (MessageId::LENGTH + 2 * core::mem::size_of::<u8>()) as u32 }>;

fn unpack_prefix_to_conflict_validation_error(
    err: UnpackPrefixError<MessageUnpackError, <ConflictsCount as TryFrom<u32>>::Error>,
) -> MessageUnpackError {
    err.unwrap_packable_or_else(|prefix_err| ValidationError::InvalidConflictsCount(prefix_err.into()))
}

fn unpack_prefix_to_timestamp_validation_error(
    err: UnpackPrefixError<MessageUnpackError, <TimestampsCount as TryFrom<u32>>::Error>,
) -> MessageUnpackError {
    err.unwrap_packable_or_else(|prefix_err| ValidationError::InvalidTimestampsCount(prefix_err.into()))
}

/// Payload describing opinions on conflicts and timestamps of messages.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct FpcPayload {
    /// Collection of opinions on conflicting transactions.
    #[packable(unpack_error_with = unpack_prefix_to_conflict_validation_error)]
    conflicts: VecPrefix<Conflict, ConflictsCount>,
    /// Collection of opinions on message timestamps.
    #[packable(unpack_error_with = unpack_prefix_to_timestamp_validation_error)]
    timestamps: VecPrefix<Timestamp, TimestampsCount>,
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

    /// Returns the [`Conflict`]s of an [`FpcPayload`].
    pub fn conflicts(&self) -> impl Iterator<Item = &Conflict> {
        self.conflicts.iter()
    }

    /// Returns the [`Timestamp`]s of an [`FpcPayload`].
    pub fn timestamps(&self) -> impl Iterator<Item = &Timestamp> {
        self.timestamps.iter()
    }
}

/// A builder to build an [`FpcPayload`].
#[derive(Default)]
pub struct FpcPayloadBuilder {
    conflicts: Vec<Conflict>,
    timestamps: Vec<Timestamp>,
}

impl FpcPayloadBuilder {
    /// Creates a new [`FpcPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a collection of conflicts to the [`FpcPayloadBuilder`].
    pub fn with_conflicts(mut self, conflicts: Vec<Conflict>) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Adds a collection of timestamps to the [`FpcPayloadBuilder`].
    pub fn with_timestamps(mut self, timestamps: Vec<Timestamp>) -> Self {
        self.timestamps = timestamps;
        self
    }

    /// Finishes an [`FpcPayloadBuilder`] into an [`FpcPayload`].
    pub fn finish(self) -> Result<FpcPayload, ValidationError> {
        Ok(FpcPayload {
            conflicts: self
                .conflicts
                .try_into()
                .map_err(ValidationError::InvalidConflictsCount)?,
            timestamps: self
                .timestamps
                .try_into()
                .map_err(ValidationError::InvalidTimestampsCount)?,
        })
    }
}
