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

use core::{convert::Infallible, fmt, ops::RangeInclusive};

/// The maximum number of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_MAX: u16 = INPUT_COUNT_MAX;
/// The range of valid numbers of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_RANGE: RangeInclusive<u16> = 1..=UNLOCK_BLOCK_COUNT_MAX; // [1..127]
/// The maximum index of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_MAX: u16 = UNLOCK_BLOCK_COUNT_MAX - 1; // [0..126]
/// The range of valid indices of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_RANGE: RangeInclusive<u16> = 0..=UNLOCK_BLOCK_INDEX_MAX; // [0..126]

/// Error encountered unpacking an [`UnlockBlock`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum UnlockBlockUnpackError {
    InvalidKind(u8),
    ValidationError(ValidationError),
}

impl fmt::Display for UnlockBlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid UnlockBlock kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = UnlockBlockUnpackError::InvalidKind)]
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
    /// Returns the unlock block kind of an [`UnlockBlock`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlock::KIND,
            Self::Reference(_) => ReferenceUnlock::KIND,
        }
    }
}
