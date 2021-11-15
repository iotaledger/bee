// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{MessageUnpackError, ValidationError},
    output::OUTPUT_INDEX_MAX,
    payload::transaction::TransactionId,
    util::hex_decode,
};

use bee_packable::{
    bounded::{BoundedU16, InvalidBoundedU16},
    Packable,
};

use core::{convert::Infallible, fmt};

/// Error encountered unpacking an [`OutputId`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum OutputIdUnpackError {
    Validation(ValidationError),
}

impl_wrapped_variant!(OutputIdUnpackError, OutputIdUnpackError::Validation, ValidationError);

impl From<InvalidBoundedU16<0, OUTPUT_INDEX_MAX>> for ValidationError {
    fn from(err: InvalidBoundedU16<0, OUTPUT_INDEX_MAX>) -> Self {
        ValidationError::InvalidOutputIndex(err.0)
    }
}

impl From<Infallible> for ValidationError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}

impl fmt::Display for OutputIdUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "{}", e),
        }
    }
}

/// The identifier of an [`Output`](crate::output::Output).
///
/// An [`OutputId`] must:
/// * Have an index that falls within [`OUTPUT_INDEX_RANGE`](crate::output::OUTPUT_INDEX_RANGE).
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError, with = ValidationError::from)]
pub struct OutputId {
    transaction_id: TransactionId,
    index: BoundedU16<0, OUTPUT_INDEX_MAX>,
}

impl OutputId {
    /// The length of an [`OutputId`].
    pub const LENGTH: usize = TransactionId::LENGTH + core::mem::size_of::<u16>();

    /// Creates a new [`OutputId`].
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, ValidationError> {
        Ok(Self {
            transaction_id,
            index: index.try_into()?,
        })
    }

    /// Returns the [`TransactionId`] of an [`OutputId`].
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an [`OutputId`].
    pub fn index(&self) -> u16 {
        self.index.into()
    }

    /// Splits an [`OutputId`] into its [`TransactionId`] and index.
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index.into())
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

impl core::str::FromStr for OutputId {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        OutputId::try_from(hex_decode(hex)?)
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.transaction_id, hex::encode(self.index().to_le_bytes()))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}
