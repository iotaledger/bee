// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod nft;
mod reference;
mod signature;

pub(crate) use alias::AliasIndex;
pub use alias::AliasUnlockBlock;
pub(crate) use nft::NftIndex;
pub use nft::NftUnlockBlock;
pub(crate) use reference::ReferenceIndex;
pub use reference::ReferenceUnlockBlock;
pub use signature::SignatureUnlockBlock;

use crate::{
    input::{INPUT_COUNT_MAX, INPUT_COUNT_RANGE, INPUT_INDEX_MAX, INPUT_INDEX_RANGE},
    Error,
};

use bee_packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

use derive_more::{Deref, From};

use core::ops::RangeInclusive;
use std::collections::HashSet;

/// The maximum number of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_MAX: u16 = INPUT_COUNT_MAX; //127
/// The range of valid numbers of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_RANGE: RangeInclusive<u16> = INPUT_COUNT_RANGE; // [1..127]
/// The maximum index of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_MAX: u16 = INPUT_INDEX_MAX; // 126
/// The range of valid indices of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_RANGE: RangeInclusive<u16> = INPUT_INDEX_RANGE; // [0..126]

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, From, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidUnlockBlockKind)]
pub enum UnlockBlock {
    /// A signature unlock block.
    #[packable(tag = SignatureUnlockBlock::KIND)]
    Signature(SignatureUnlockBlock),
    /// A reference unlock block.
    #[packable(tag = ReferenceUnlockBlock::KIND)]
    Reference(ReferenceUnlockBlock),
    /// An alias unlock block.
    #[packable(tag = AliasUnlockBlock::KIND)]
    Alias(AliasUnlockBlock),
    /// An NFT unlock block.
    #[packable(tag = NftUnlockBlock::KIND)]
    Nft(NftUnlockBlock),
}

impl UnlockBlock {
    /// Returns the unlock kind of an `UnlockBlock`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Signature(_) => SignatureUnlockBlock::KIND,
            Self::Reference(_) => ReferenceUnlockBlock::KIND,
            Self::Alias(_) => AliasUnlockBlock::KIND,
            Self::Nft(_) => NftUnlockBlock::KIND,
        }
    }
}

pub(crate) type UnlockBlockCount =
    BoundedU16<{ *UNLOCK_BLOCK_COUNT_RANGE.start() }, { *UNLOCK_BLOCK_COUNT_RANGE.end() }>;

/// A collection of unlock blocks.
#[derive(Clone, Debug, Eq, PartialEq, Deref, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_packable_or_else(|p| Error::InvalidUnlockBlockCount(p.into())))]
pub struct UnlockBlocks(
    #[packable(verify_with = validate_unlock_blocks)] BoxedSlicePrefix<UnlockBlock, UnlockBlockCount>,
);

impl UnlockBlocks {
    /// Creates a new `UnlockBlocks`.
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, Error> {
        let unlock_blocks: BoxedSlicePrefix<UnlockBlock, UnlockBlockCount> = unlock_blocks
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidUnlockBlockCount)?;

        validate_unlock_blocks::<true>(&unlock_blocks)?;

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

fn validate_unlock_blocks<const VERIFY: bool>(unlock_blocks: &[UnlockBlock]) -> Result<(), Error> {
    let mut seen_signatures = HashSet::new();

    for (index, unlock_block) in (0u16..).zip(unlock_blocks.iter()) {
        match unlock_block {
            UnlockBlock::Signature(signature) => {
                if !seen_signatures.insert(signature) {
                    return Err(Error::DuplicateSignatureUnlockBlock(index));
                }
            }
            UnlockBlock::Reference(reference) => {
                if index == 0
                    || reference.index() >= index as u16
                    || matches!(unlock_blocks[reference.index() as usize], UnlockBlock::Reference(_))
                {
                    return Err(Error::InvalidUnlockBlockReference(index));
                }
            }
            UnlockBlock::Alias(alias) => {
                if index == 0 || alias.index() >= index as u16 {
                    return Err(Error::InvalidUnlockBlockAlias(index));
                }
            }
            UnlockBlock::Nft(nft) => {
                if index == 0 || nft.index() >= index as u16 {
                    return Err(Error::InvalidUnlockBlockNft(index));
                }
            }
        }
    }

    Ok(())
}
