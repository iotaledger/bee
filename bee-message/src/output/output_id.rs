// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{MessageUnpackError, ValidationError},
    output::OUTPUT_INDEX_RANGE,
    payload::transaction::TransactionId,
    util::hex_decode,
};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker, coerce::*};

use core::{
    convert::{From, Infallible, TryFrom, TryInto},
    fmt,
    str::FromStr,
};

/// Error encountered unpacking an [`OutputId`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum OutputIdUnpackError {
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    OutputIdUnpackError,
    ValidationError,
    OutputIdUnpackError::ValidationError
);

impl fmt::Display for OutputIdUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// The identifier of an [`Output`](crate::output::Output).
///
/// An [`OutputId`] must:
/// * Have an index that falls within [`OUTPUT_INDEX_RANGE`].
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct OutputId {
    transaction_id: TransactionId,
    index: u16,
}

impl OutputId {
    /// The length of an [`OutputId`].
    pub const LENGTH: usize = TransactionId::LENGTH + core::mem::size_of::<u16>();

    /// Creates a new [`OutputId`].
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, ValidationError> {
        validate_index(index)?;

        Ok(Self { transaction_id, index })
    }

    /// Returns the [`TransactionId`] of an [`OutputId`].
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an [`OutputId`].
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Splits an [`OutputId`] into its [`TransactionId`] and index.
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index)
    }
}

impl Packable for OutputId {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.transaction_id.packed_len() + self.index.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.transaction_id.pack(packer).infallible()?;
        self.index.pack(packer).infallible()?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let transaction_id = TransactionId::unpack(unpacker).infallible()?;

        let index = u16::unpack(unpacker).infallible()?;
        validate_index(index).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { transaction_id, index })
    }
}

impl TryFrom<[u8; OutputId::LENGTH]> for OutputId {
    type Error = ValidationError;

    fn try_from(bytes: [u8; OutputId::LENGTH]) -> Result<Self, Self::Error> {
        let (transaction_id, index) = bytes.split_at(TransactionId::LENGTH);

        Self::new(
            // Unwrap is fine because size is already known and valid.
            From::<[u8; TransactionId::LENGTH]>::from(transaction_id.try_into().unwrap()),
            // Unwrap is fine because size is already known and valid.
            u16::from_le_bytes(index.try_into().unwrap()),
        )
    }
}

impl FromStr for OutputId {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        OutputId::try_from(hex_decode(hex)?)
    }
}

fn validate_index(index: u16) -> Result<(), ValidationError> {
    if !OUTPUT_INDEX_RANGE.contains(&index) {
        Err(ValidationError::InvalidOutputIndex(index))
    } else {
        Ok(())
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.transaction_id, hex::encode(self.index.to_le_bytes()))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}
