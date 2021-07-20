// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;
mod unlock_blocks;

pub use reference::{ReferenceUnlock, ReferenceUnlockUnpackError};
pub use unlock_blocks::{UnlockBlocks, UnlockBlocksPackError, UnlockBlocksUnpackError};

use crate::{input::INPUT_COUNT_MAX, signature::SignatureUnlock, MessageUnpackError, ValidationError};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

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

/// Error encountered unpacking an `UnlockBlock`.
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
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "enable-serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum UnlockBlock {
    /// A signature unlock block.
    Signature(SignatureUnlock),
    /// A reference unlock block.
    Reference(ReferenceUnlock),
}

impl_wrapped_variant!(UnlockBlock, SignatureUnlock, UnlockBlock::Signature);
impl_wrapped_variant!(UnlockBlock, ReferenceUnlock, UnlockBlock::Reference);

impl UnlockBlock {
    /// Returns the unlock kind of an `UnlockBlock`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlock::KIND,
            Self::Reference(_) => ReferenceUnlock::KIND,
        }
    }
}

impl Packable for UnlockBlock {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Self::Signature(signature) => signature.packed_len(),
                Self::Reference(reference) => reference.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).map_err(PackError::infallible)?;

        match self {
            Self::Signature(signature) => signature.pack(packer).map_err(PackError::infallible)?,
            Self::Reference(reference) => reference.pack(packer).map_err(PackError::infallible)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let kind = u8::unpack(unpacker).map_err(UnpackError::infallible)?;

        let variant = match kind {
            SignatureUnlock::KIND => Self::Signature(SignatureUnlock::unpack(unpacker)?),
            ReferenceUnlock::KIND => Self::Reference(ReferenceUnlock::unpack(unpacker).map_err(UnpackError::coerce)?),
            tag => {
                return Err(UnpackError::Packable(
                    UnlockBlockUnpackError::InvalidUnlockBlockKind(tag).into(),
                ));
            }
        };

        Ok(variant)
    }
}
