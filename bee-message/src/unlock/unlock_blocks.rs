// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock, UnlockBlockUnpackError, UNLOCK_BLOCK_COUNT_RANGE},
    MessagePackError, MessageUnpackError, ValidationError,
};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use hashbrown::HashSet;

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
    ops::Deref,
};

const PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX: usize = *UNLOCK_BLOCK_COUNT_RANGE.end();

/// Error encountered while packing [`UnlockBlocks`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum UnlockBlocksPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u16>> for UnlockBlocksPackError {
    fn from(error: PackPrefixError<Infallible, u16>) -> Self {
        match error {
            PackPrefixError::Packable(error) => match error {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for UnlockBlocksPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix"),
        }
    }
}

/// Error encountered while unpacking [`UnlockBlocks`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum UnlockBlocksUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
    UnlockBlockUnpack(UnlockBlockUnpackError),
    ValidationError(ValidationError),
}

impl_wrapped_variant!(
    UnlockBlocksUnpackError,
    ValidationError,
    UnlockBlocksUnpackError::ValidationError
);

impl From<UnpackPrefixError<UnlockBlockUnpackError, u16>> for UnlockBlocksUnpackError {
    fn from(error: UnpackPrefixError<UnlockBlockUnpackError, u16>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(error) => Self::from(error),
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl From<UnlockBlockUnpackError> for UnlockBlocksUnpackError {
    fn from(error: UnlockBlockUnpackError) -> Self {
        match error {
            UnlockBlockUnpackError::ValidationError(error) => Self::ValidationError(error),
            error => Self::UnlockBlockUnpack(error),
        }
    }
}

impl fmt::Display for UnlockBlocksUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix"),
            Self::InvalidPrefixLength(len) => write!(f, "unpacked prefix larger than maximum specified: {}", len),
            Self::UnlockBlockUnpack(e) => write!(f, "{}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// A collection of unlock blocks.
///
/// An [`UnlockBlocks`] collection must:
/// * Contain a number of [`UnlockBlock`]s within [`UNLOCK_BLOCK_COUNT_RANGE`].
/// * Ensure all signatures in [`SignatureUnlock`](crate::unlock::SignatureUnlock) blocks are unique across the collection.
/// * Ensure [`ReferenceUnlock`](crate::unlock::ReferenceUnlock) blocks specify a previous existing.
/// [`SignatureUnlock`](crate::unlock::SignatureUnlock) block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct UnlockBlocks(Vec<UnlockBlock>);

impl UnlockBlocks {
    /// Creates a new [`UnlockBlocks`].
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, ValidationError> {
        validate_unlock_block_count(unlock_blocks.len())?;
        validate_unlock_block_variants(&unlock_blocks)?;

        Ok(Self(unlock_blocks))
    }

    /// Gets an [`UnlockBlock`] from an [`UnlockBlocks`].
    /// Returns the referenced unlock block if the requested unlock block was a reference.
    pub fn get(&self, index: usize) -> Option<&UnlockBlock> {
        match self.0.get(index) {
            Some(UnlockBlock::Reference(reference)) => self.0.get(reference.index() as usize),
            Some(unlock_block) => Some(unlock_block),
            None => None,
        }
    }
}

impl Deref for UnlockBlocks {
    type Target = [UnlockBlock];

    fn deref(&self) -> &Self::Target {
        &self.0.as_slice()
    }
}

impl Packable for UnlockBlocks {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwrap is safe, since UnlockBlock count is already validated.
        let prefixed: VecPrefix<UnlockBlock, u16, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX> =
            self.0.clone().try_into().unwrap();
        prefixed.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // Unwrap is safe, since UnlockBlock count is already validated.
        let prefixed: VecPrefix<UnlockBlock, u16, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX> =
            self.0.clone().try_into().unwrap();
        prefixed
            .pack(packer)
            .map_err(PackError::coerce::<UnlockBlocksPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let inner_prefixed = VecPrefix::<UnlockBlock, u16, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX>::unpack(unpacker);

        let inner: Vec<UnlockBlock> = if let Err(unpack_err) = inner_prefixed {
            match unpack_err {
                UnpackError::Packable(e) => match e {
                    UnpackPrefixError::InvalidPrefixLength(len) => {
                        return Err(UnpackError::Packable(
                            UnlockBlocksUnpackError::InvalidPrefixLength(len).into(),
                        ));
                    }
                    UnpackPrefixError::Packable(err) => return Err(UnpackError::Packable(err)),
                    UnpackPrefixError::Prefix(_) => {
                        return Err(UnpackError::Packable(UnlockBlocksUnpackError::InvalidPrefix.into()));
                    }
                },
                UnpackError::Unpacker(e) => return Err(UnpackError::Unpacker(e)),
            }
        } else {
            // Unwrap is fine, we have just checked that this value is `Ok`.
            inner_prefixed.ok().unwrap().into()
        };

        validate_unlock_block_count(inner.len()).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_unlock_block_variants(&inner).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self(inner))
    }
}

fn validate_unlock_block_count(len: usize) -> Result<(), ValidationError> {
    if !UNLOCK_BLOCK_COUNT_RANGE.contains(&len) {
        Err(ValidationError::InvalidUnlockBlockCount(len))
    } else {
        Ok(())
    }
}

fn validate_unlock_block_variants(unlock_blocks: &[UnlockBlock]) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();

    for (idx, unlock_block) in unlock_blocks.iter().enumerate() {
        let signature = validate_unlock_block_variant(idx, unlock_block, &unlock_blocks)?;

        if let Some(signature) = signature {
            if !seen.insert(signature) {
                return Err(ValidationError::DuplicateSignature(idx));
            }
        }
    }

    Ok(())
}

fn validate_unlock_block_variant<'a>(
    idx: usize,
    unlock_block: &'a UnlockBlock,
    unlock_blocks: &'a [UnlockBlock],
) -> Result<Option<&'a SignatureUnlock>, ValidationError> {
    match unlock_block {
        UnlockBlock::Reference(r) => validate_unlock_block_reference(&r, idx, unlock_blocks).map(|_| None),
        UnlockBlock::Signature(s) => Ok(Some(s)),
    }
}

fn validate_unlock_block_reference(
    reference: &ReferenceUnlock,
    idx: usize,
    unlock_blocks: &[UnlockBlock],
) -> Result<(), ValidationError> {
    let r_idx = reference.index();

    if idx == 0 || r_idx >= idx as u16 || matches!(unlock_blocks[r_idx as usize], UnlockBlock::Reference(_)) {
        Err(ValidationError::InvalidUnlockBlockReference(idx))
    } else {
        Ok(())
    }
}
