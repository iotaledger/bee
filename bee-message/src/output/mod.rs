// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of outputs.

mod output_id;
mod signature_locked_asset;
mod signature_locked_single;

pub use crate::{
    error::{MessagePackError, MessageUnpackError, ValidationError},
    input::INPUT_COUNT_MAX,
};

pub use output_id::{OutputId, OutputIdUnpackError};
pub use signature_locked_asset::{
    AssetBalance, AssetId, SignatureLockedAssetOutput, SignatureLockedAssetPackError, SignatureLockedAssetUnpackError,
};
pub use signature_locked_single::{
    SignatureLockedSingleOutput, SignatureLockedSingleUnpackError, SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT,
};

use bee_packable::{coerce::*, PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker};

use core::{
    fmt,
    ops::{Range, RangeInclusive},
};

/// The maximum number of outputs for a transaction.
pub const OUTPUT_COUNT_MAX: usize = INPUT_COUNT_MAX;
/// The range of valid numbers of outputs for a transaction [1..127].
pub const OUTPUT_COUNT_RANGE: RangeInclusive<usize> = 1..=OUTPUT_COUNT_MAX;
/// The valid range of indices for outputs for a transaction [0..126].
pub const OUTPUT_INDEX_RANGE: Range<u16> = 0..OUTPUT_COUNT_MAX as u16;

/// Error encountered unpacking a transaction output.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum OutputUnpackError {
    InvalidAddressKind(u8),
    InvalidOutputKind(u8),
    ValidationError(ValidationError),
}

impl From<UnknownTagError<u8>> for OutputUnpackError {
    fn from(error: UnknownTagError<u8>) -> Self {
        Self::InvalidAddressKind(error.0)
    }
}

impl fmt::Display for OutputUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddressKind(kind) => write!(f, "invalid Address kind: {}", kind),
            Self::InvalidOutputKind(kind) => write!(f, "invalid Output kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Output {
    /// A signature locked single output.
    SignatureLockedSingle(SignatureLockedSingleOutput),
    /// A signature locked asset output.
    SignatureLockedAsset(SignatureLockedAssetOutput),
}

impl_wrapped_variant!(Output, SignatureLockedSingleOutput, Output::SignatureLockedSingle);

impl Output {
    /// Returns the output kind of an [`Output`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::SignatureLockedSingle(_) => SignatureLockedSingleOutput::KIND,
            Self::SignatureLockedAsset(_) => SignatureLockedAssetOutput::KIND,
        }
    }
}

impl Packable for Output {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Self::SignatureLockedSingle(output) => output.packed_len(),
                Self::SignatureLockedAsset(output) => output.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).infallible()?;

        match self {
            Self::SignatureLockedSingle(output) => output.pack(packer).infallible()?,
            Self::SignatureLockedAsset(output) => output.pack(packer)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let kind = u8::unpack(unpacker).infallible()?;

        let variant = match kind {
            SignatureLockedSingleOutput::KIND => {
                Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(unpacker)?)
            }
            SignatureLockedAssetOutput::KIND => {
                Self::SignatureLockedAsset(SignatureLockedAssetOutput::unpack(unpacker)?)
            }
            tag => Err(UnpackError::Packable(OutputUnpackError::InvalidOutputKind(tag))).coerce()?,
        };

        Ok(variant)
    }
}
