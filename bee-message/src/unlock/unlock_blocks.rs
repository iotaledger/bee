// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    unlock::{ReferenceUnlock, SignatureUnlock, UnlockBlock, UNLOCK_BLOCK_COUNT_RANGE},
    MessageUnpackError, ValidationError,
};

use bee_packable::{bounded::BoundedU16, prefix::VecPrefix, Packable};

use hashbrown::HashSet;

use alloc::vec::Vec;
use core::ops::Deref;

pub(crate) type UnlockBlockCount =
    BoundedU16<{ *UNLOCK_BLOCK_COUNT_RANGE.start() }, { *UNLOCK_BLOCK_COUNT_RANGE.end() }>;

/// A collection of unlock blocks.
///
/// An [`UnlockBlocks`] collection must:
/// * Contain a number of [`UnlockBlock`]s within [`UNLOCK_BLOCK_COUNT_RANGE`].
/// * Ensure all signatures in [`SignatureUnlock`](crate::unlock::SignatureUnlock) blocks are unique across the
///   collection.
/// * Ensure [`ReferenceUnlock`](crate::unlock::ReferenceUnlock) blocks specify a previous existing.
/// [`SignatureUnlock`](crate::unlock::SignatureUnlock) block.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct UnlockBlocks(
    #[packable(verify_with = validate_unlock_block_variants)]
    #[packable(unpack_error_with = |e| e.unwrap_packable_or_else(|p| ValidationError::InvalidUnlockBlockCount(p.into())))]
    VecPrefix<UnlockBlock, UnlockBlockCount>,
);

impl UnlockBlocks {
    /// Creates a new [`UnlockBlocks`].
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, ValidationError> {
        let unlock_blocks = VecPrefix::<UnlockBlock, UnlockBlockCount>::try_from(unlock_blocks)
            .map_err(ValidationError::InvalidUnlockBlockCount)?;

        validate_unlock_block_variants(&unlock_blocks)?;

        Ok(Self(unlock_blocks))
    }

    /// Gets an [`UnlockBlock`] from an [`UnlockBlockbee-common/bee-packable/src/packable/bounded.rss`].
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
        self.0.as_slice()
    }
}

fn validate_unlock_block_variants(unlock_blocks: &[UnlockBlock]) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();

    for (idx, unlock_block) in unlock_blocks.iter().enumerate() {
        let signature = validate_unlock_block_variant(idx, unlock_block, unlock_blocks)?;

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
        UnlockBlock::Reference(reference) => {
            validate_unlock_block_reference(reference, idx, unlock_blocks).map(|_| None)
        }
        UnlockBlock::Signature(signature) => Ok(Some(signature)),
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
