// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the FPC statement payload.

mod conflicts;
mod timestamps;

pub use conflicts::{Conflict, Conflicts};
pub use timestamps::{Timestamp, Timestamps};

use crate::{MessagePackError, MessageUnpackError, ValidationError};

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
                write!(f, "unpacked conflicts/timestamps prefix larger than maximum specified: {}", len)
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FpcPayload {
    /// Version of the FPC statement payload.
    version: u8,
    /// Collection of opinions on conflicting transactions.
    conflicts: Conflicts,
    /// Collection of opinions on message timestamps.
    timestamps: Timestamps,
}

impl FpcPayload {
    /// The payload kind of an `FpcPayload`.
    pub const KIND: u32 = 2;

    /// Returns a new `FpcPayloadBuilder` in order to build an `FpcPayload`.
    pub fn builder() -> FpcPayloadBuilder {
        FpcPayloadBuilder::new()
    }
}

impl Packable for FpcPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len() + self.conflicts.packed_len() + self.timestamps.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
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

        let conflicts = Conflicts::unpack(unpacker)
            .map_err(UnpackError::coerce::<FpcUnpackError>)
            .map_err(UnpackError::coerce)?;

        let timestamps = Timestamps::unpack(unpacker)
            .map_err(UnpackError::coerce::<FpcUnpackError>)
            .map_err(UnpackError::coerce)?;

        Ok(Self {
            version,
            conflicts,
            timestamps,
        })
    }
}

/// A builder to build an `FpcPayload`.
#[derive(Default)]
pub struct FpcPayloadBuilder {
    version: Option<u8>,
    conflicts: Conflicts,
    timestamps: Timestamps,
}

impl FpcPayloadBuilder {
    /// Creates a new `FpcPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a version number to the `FpcPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version.replace(version);
        self
    }

    /// Adds a collection of conflicts to the `FpcPayloadBuilder`.
    pub fn with_conflicts(mut self, conflicts: Conflicts) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Adds a collection of timestamps to the `FpcPayloadBuilder`.
    pub fn with_timestamps(mut self, timestamps: Timestamps) -> Self {
        self.timestamps = timestamps;
        self
    }

    /// Finishes an `FpcPayloadBuilder` into an `FpcPayload`.
    pub fn finish(self) -> Result<FpcPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;

        Ok(FpcPayload {
            version,
            conflicts: self.conflicts,
            timestamps: self.timestamps,
        })
    }
}
