// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;

pub use reference::ReferenceUnlock;

use crate::{
    constants::UNLOCK_BLOCK_COUNT_RANGE, signature::SignatureUnlock, unlock::reference::ReferenceUnlockUnpackError,
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
            Self::InvalidUnlockBlockKind(kind) => write!(f, "Invalid unlock block kind: {}", kind),
            Self::ValidationError(e) => write!(f, "{}", e),
        }
    }
}

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
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

/// Error encountered while packing `UnlockBlocks`.
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

/// Error encountered while unpacking `UnlockBlocks`.
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
/// An `UnlockBlocks` collection must:
/// * Contain a number of `UnlockBlock`s within `UNLOCK_BLOCKS_COUNT_RANGE`.
/// * Ensure all signatures in `Signature` blocks are unique across the collection.
/// * Ensure `Reference` blocks specify a previous existing `Signature` block.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnlockBlocks {
    inner: Vec<UnlockBlock>,
}

impl UnlockBlocks {
    /// Creates a new `UnlockBlocks`.
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, ValidationError> {
        validate_unlock_block_count(unlock_blocks.len())?;
        validate_unlock_block_variants(&unlock_blocks)?;

        Ok(Self { inner: unlock_blocks })
    }

    /// Gets an `UnlockBlock` from an `UnlockBlocks`.
    /// Returns the referenced unlock block if the requested unlock block was a reference.
    pub fn get(&self, index: usize) -> Option<&UnlockBlock> {
        match self.inner.get(index) {
            Some(UnlockBlock::Reference(reference)) => self.inner.get(reference.index() as usize),
            Some(unlock_block) => Some(unlock_block),
            None => None,
        }
    }
}

impl Deref for UnlockBlocks {
    type Target = [UnlockBlock];

    fn deref(&self) -> &Self::Target {
        &self.inner.as_slice()
    }
}

impl Packable for UnlockBlocks {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwrap is safe, since UnlockBlock count is already validated.
        let prefixed: VecPrefix<UnlockBlock, u16, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX> =
            self.inner.clone().try_into().unwrap();
        prefixed.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // Unwrap is safe, since UnlockBlock count is already validated.
        let prefixed: VecPrefix<UnlockBlock, u16, PREFIXED_UNLOCK_BLOCKS_LENGTH_MAX> =
            self.inner.clone().try_into().unwrap();
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
            inner_prefixed.ok().unwrap().into()
        };

        validate_unlock_block_count(inner.len()).map_err(|e| UnpackError::Packable(e.into()))?;
        validate_unlock_block_variants(&inner).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(Self { inner })
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

fn validate_unlock_block_variant(
    idx: usize,
    unlock_block: &UnlockBlock,
    unlock_blocks: &[UnlockBlock],
) -> Result<Option<SignatureUnlock>, ValidationError> {
    match unlock_block {
        UnlockBlock::Reference(r) => validate_unlock_block_reference(&r, idx, unlock_blocks).map(|_| None),
        UnlockBlock::Signature(s) => Ok(Some(s.clone())),
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
