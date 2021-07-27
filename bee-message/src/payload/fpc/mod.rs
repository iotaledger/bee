// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the FPC statement payload.

mod conflicts;
mod timestamps;

pub use conflicts::{Conflict, Conflicts};
pub use timestamps::{Timestamp, Timestamps};

use crate::{payload::MessagePayload, MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker,
};

use core::{convert::Infallible, fmt};

/// Error encountered packing FPC payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum FpcPackError {
    InvalidPrefix,
}

impl_from_infallible!(FpcPackError);

impl From<PackPrefixError<Infallible, u32>> for FpcPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for FpcPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for conflicts/timestamps"),
        }
    }
}

/// Error encountered unpacking FPC payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum FpcUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
}

impl_from_infallible!(FpcUnpackError);

impl fmt::Display for FpcUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for conflicts/timestamps"),
            Self::InvalidPrefixLength(len) => {
                write!(
                    f,
                    "unpacked conflicts/timestamps prefix larger than maximum specified: {}",
                    len
                )
            }
        }
    }
}

impl From<UnpackPrefixError<Infallible, u32>> for FpcUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

/// Payload describing opinions on conflicts and timestamps of messages.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
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

impl Packable for FpcPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        Self::VERSION.packed_len() + self.conflicts.packed_len() + self.timestamps.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        Self::VERSION.pack(packer).map_err(PackError::infallible)?;
        self.conflicts
            .pack(packer)
            .map_err(PackError::coerce::<FpcPackError>)
            .map_err(PackError::coerce)?;
        self.timestamps
            .pack(packer)
            .map_err(PackError::coerce::<FpcPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_payload_version(version).map_err(|e| UnpackError::Packable(e.into()))?;

        let conflicts = Conflicts::unpack(unpacker)
            .map_err(UnpackError::coerce::<FpcUnpackError>)
            .map_err(UnpackError::coerce)?;

        let timestamps = Timestamps::unpack(unpacker)
            .map_err(UnpackError::coerce::<FpcUnpackError>)
            .map_err(UnpackError::coerce)?;

        Ok(Self { conflicts, timestamps })
    }
}

fn validate_payload_version(version: u8) -> Result<(), ValidationError> {
    if version != FpcPayload::VERSION {
        Err(ValidationError::InvalidPayloadVersion(version, FpcPayload::KIND))
    } else {
        Ok(())
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
