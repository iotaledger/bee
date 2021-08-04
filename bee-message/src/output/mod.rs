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
pub use signature_locked_asset::{AssetBalance, AssetId, SignatureLockedAssetOutput, SignatureLockedAssetUnpackError};
pub use signature_locked_single::{
    SignatureLockedSingleOutput, SignatureLockedSingleUnpackError, SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT,
};

use bee_packable::Packable;

use core::{convert::Infallible, fmt, ops::RangeInclusive};

/// The maximum number of outputs of a transaction.
pub const OUTPUT_COUNT_MAX: u16 = INPUT_COUNT_MAX;
/// The range of valid numbers of outputs of a transaction .
pub const OUTPUT_COUNT_RANGE: RangeInclusive<u16> = 1..=OUTPUT_COUNT_MAX; //[1..127]
/// The maximum index of outputs of a transaction.
pub const OUTPUT_INDEX_MAX: u16 = OUTPUT_COUNT_MAX - 1;
/// The range of valid indices of outputs of a transaction .
pub const OUTPUT_INDEX_RANGE: RangeInclusive<u16> = 0..=OUTPUT_COUNT_MAX; //[0..126]

/// Error encountered unpacking an [`Output`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum OutputUnpackError {
    InvalidKind(u8),
    ValidationError(ValidationError),
}

impl fmt::Display for OutputUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid Output kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A generic output supporting different output kinds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = OutputUnpackError::InvalidKind)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = MessageUnpackError)]
pub enum Output {
    /// A signature locked single output.
    #[packable(tag = SignatureLockedSingleOutput::KIND)]
    SignatureLockedSingle(SignatureLockedSingleOutput),
    /// A signature locked asset output.
    #[packable(tag = SignatureLockedAssetOutput::KIND)]
    SignatureLockedAsset(SignatureLockedAssetOutput),
}

impl_wrapped_variant!(Output, SignatureLockedSingleOutput, Output::SignatureLockedSingle);
impl_wrapped_variant!(Output, SignatureLockedAssetOutput, Output::SignatureLockedAsset);

impl Output {
    /// Returns the output kind of an [`Output`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::SignatureLockedSingle(_) => SignatureLockedSingleOutput::KIND,
            Self::SignatureLockedAsset(_) => SignatureLockedAssetOutput::KIND,
        }
    }
}
