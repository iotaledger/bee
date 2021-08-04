// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod output_id;
mod signature_locked_asset_allowance;
mod signature_locked_single;

pub use crate::error::{MessageUnpackError, ValidationError};

pub use output_id::{OutputId, OutputIdUnpackError, OUTPUT_ID_LENGTH};
pub use signature_locked_asset_allowance::{
    AssetBalance, SignatureLockedAssetOutput, SignatureLockedAssetPackError, SignatureLockedAssetUnpackError,
};
pub use signature_locked_single::{
    SignatureLockedSingleOutput, SignatureLockedSingleUnpackError, SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT,
};

use bee_packable::{PackError, Packable, Packer, UnknownTagError, UnpackError, Unpacker};

use core::{convert::Infallible, fmt};

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
            Self::InvalidAddressKind(kind) => write!(f, "Invalid address kind: {}", kind),
            Self::InvalidOutputKind(kind) => write!(f, "Invalid output kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A generic output that can represent different types defining the deposit of funds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Output {
    /// A signature locked single output.
    SignatureLockedSingle(SignatureLockedSingleOutput),
}

impl_wrapped_variant!(Output, SignatureLockedSingleOutput, Output::SignatureLockedSingle);

impl Output {
    /// Return the output kind of an `Output`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::SignatureLockedSingle(_) => SignatureLockedSingleOutput::KIND,
        }
    }
}

impl Packable for Output {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Self::SignatureLockedSingle(output) => output.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).map_err(PackError::infallible)?;

        match self {
            Self::SignatureLockedSingle(output) => output.pack(packer).map_err(PackError::infallible)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let kind = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let variant = match kind {
            SignatureLockedSingleOutput::KIND => {
                Self::SignatureLockedSingle(SignatureLockedSingleOutput::unpack(unpacker)?)
            }
            tag => {
                Err(UnpackError::Packable(OutputUnpackError::InvalidOutputKind(tag))).map_err(UnpackError::coerce)?
            }
        };

        Ok(variant)
    }
}
