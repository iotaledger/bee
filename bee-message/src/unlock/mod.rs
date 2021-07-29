// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of unlock blocks.

mod reference;
mod signature;
mod unlock_blocks;

pub use reference::{ReferenceUnlock, ReferenceUnlockUnpackError};
pub use signature::SignatureUnlock;
pub use unlock_blocks::{UnlockBlocks, UnlockBlocksPackError, UnlockBlocksUnpackError};

use crate::{input::INPUT_COUNT_MAX, MessageUnpackError, ValidationError};

use bee_packable::Packable;

use core::{
    convert::Infallible,
    fmt,
    ops::{Range, RangeInclusive},
};

/// The maximum number of unlock blocks for a transaction.
pub const UNLOCK_BLOCK_COUNT_MAX: usize = INPUT_COUNT_MAX;
/// The range of valid numbers of unlock blocks for a transaction [1..127].
pub const UNLOCK_BLOCK_COUNT_RANGE: RangeInclusive<usize> = 1..=UNLOCK_BLOCK_COUNT_MAX;
/// The valid range of indices for unlock blocks for a transaction [0..126].
pub const UNLOCK_BLOCK_INDEX_RANGE: Range<u16> = 0..UNLOCK_BLOCK_COUNT_MAX as u16;

/// Error encountered unpacking an [`UnlockBlock`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum UnlockBlockUnpackError {
    InvalidUnlockBlockKind(u8),
    ValidationError(ValidationError),
}

impl From<ReferenceUnlockUnpackError> for UnlockBlockUnpackError {
    fn from(error: ReferenceUnlockUnpackError) -> Self {
        match error {
            ReferenceUnlockUnpackError::ValidationError(error) => Self::ValidationError(error),
        }
    }
}

impl fmt::Display for UnlockBlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUnlockBlockKind(kind) => write!(f, "invalid unlock block kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = UnlockBlockUnpackError::InvalidUnlockBlockKind)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = MessageUnpackError)]
pub enum UnlockBlock {
    /// A signature unlock block.
    #[packable(tag = SignatureUnlock::KIND)]
    Signature(SignatureUnlock),
    /// A reference unlock block.
    #[packable(tag = ReferenceUnlock::KIND)]
    Reference(ReferenceUnlock),
}

impl_wrapped_variant!(UnlockBlock, SignatureUnlock, UnlockBlock::Signature);
impl_wrapped_variant!(UnlockBlock, ReferenceUnlock, UnlockBlock::Reference);

impl UnlockBlock {
    /// Returns the unlock kind of an [`UnlockBlock`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlock::KIND,
            Self::Reference(_) => ReferenceUnlock::KIND,
        }
    }
}
