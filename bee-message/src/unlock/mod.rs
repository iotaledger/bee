// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;

pub use reference::ReferenceUnlock;

use crate::{
    constants::{UNLOCK_BLOCK_COUNT_MAX, UNLOCK_BLOCK_COUNT_MIN, UNLOCK_BLOCK_COUNT_RANGE},
    signature::SignatureUnlock,
    Error,
};

use bee_packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::{BoxedSlicePrefix, TryIntoPrefixError},
    unpacker::Unpacker,
    Packable,
};

use core::ops::Deref;
use std::{collections::HashSet, convert::Infallible};

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidUnlockBlockKind)]
#[packable(unpack_error = Error)]
pub enum UnlockBlock {
    /// A signature unlock block.
    #[packable(tag = SignatureUnlock::KIND)]
    Signature(SignatureUnlock),
    /// A reference unlock block.
    #[packable(tag = ReferenceUnlock::KIND)]
    Reference(ReferenceUnlock),
}

impl UnlockBlock {
    /// Returns the unlock kind of an `UnlockBlock`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlock::KIND,
            Self::Reference(_) => ReferenceUnlock::KIND,
        }
    }
}

impl From<SignatureUnlock> for UnlockBlock {
    fn from(signature: SignatureUnlock) -> Self {
        Self::Signature(signature)
    }
}

impl From<ReferenceUnlock> for UnlockBlock {
    fn from(reference: ReferenceUnlock) -> Self {
        Self::Reference(reference)
    }
}

/// A collection of unlock blocks.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct UnlockBlocks(BoxedSlicePrefix<UnlockBlock, BoundedU16<UNLOCK_BLOCK_COUNT_MIN, UNLOCK_BLOCK_COUNT_MAX>>);

impl UnlockBlocks {
    /// Creates a new `UnlockBlocks`.
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, Error> {
        let unlock_blocks =
            BoxedSlicePrefix::<_, BoundedU16<UNLOCK_BLOCK_COUNT_MIN, UNLOCK_BLOCK_COUNT_MAX>>::try_from(
                unlock_blocks.into_boxed_slice(),
            )
            .map_err(Error::InvalidUnlockBlockCount)?;

        let mut seen_signatures = HashSet::new();

        for (index, unlock_block) in unlock_blocks.iter().enumerate() {
            match unlock_block {
                UnlockBlock::Reference(r) => {
                    if index == 0
                        || r.index() >= index as u16
                        || matches!(unlock_blocks[r.index() as usize], UnlockBlock::Reference(_))
                    {
                        return Err(Error::InvalidUnlockBlockReference(index));
                    }
                }
                UnlockBlock::Signature(s) => {
                    if !seen_signatures.insert(s) {
                        return Err(Error::DuplicateSignature(index));
                    }
                }
            }
        }

        Ok(Self(unlock_blocks))
    }

    /// Gets an `UnlockBlock` from an `UnlockBlocks`.
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
        &self.0
    }
}

impl Packable for UnlockBlocks {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        (self.0.len() as u16).pack(packer)?;

        for unlock_block in self.0.as_ref() {
            unlock_block.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let unlock_blocks =
            BoxedSlicePrefix::<_, BoundedU16<UNLOCK_BLOCK_COUNT_MIN, UNLOCK_BLOCK_COUNT_MAX>>::unpack::<_, VERIFY>(
                unpacker,
            )
            .map_packable_err(|err| err.unwrap_packable_or_else(|err| Error::InvalidUnlockBlockCount(err.into())))?;

        // FIXME: avoid redundant checks.
        Self::new(Into::<Box<[_]>>::into(unlock_blocks).to_vec()).map_err(UnpackError::Packable)
    }
}
