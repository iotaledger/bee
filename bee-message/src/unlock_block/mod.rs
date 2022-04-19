// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod alias;
mod nft;
mod reference;
mod signature;

use alloc::vec::Vec;
use core::ops::RangeInclusive;

use derive_more::{Deref, From};
use hashbrown::HashSet;
use packable::{bounded::BoundedU16, prefix::BoxedSlicePrefix, Packable};

pub use self::{
    alias::AliasUnlockBlock, nft::NftUnlockBlock, reference::ReferenceUnlockBlock, signature::SignatureUnlockBlock,
};
use crate::{
    input::{INPUT_COUNT_MAX, INPUT_COUNT_RANGE, INPUT_INDEX_MAX, INPUT_INDEX_RANGE},
    Error,
};

/// The maximum number of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_MAX: u16 = INPUT_COUNT_MAX; // 128
/// The range of valid numbers of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_COUNT_RANGE: RangeInclusive<u16> = INPUT_COUNT_RANGE; // [1..128]
/// The maximum index of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_MAX: u16 = INPUT_INDEX_MAX; // 127
/// The range of valid indices of unlock blocks of a transaction.
pub const UNLOCK_BLOCK_INDEX_RANGE: RangeInclusive<u16> = INPUT_INDEX_RANGE; // [0..127]

pub(crate) type UnlockBlockIndex =
    BoundedU16<{ *UNLOCK_BLOCK_INDEX_RANGE.start() }, { *UNLOCK_BLOCK_INDEX_RANGE.end() }>;

/// Defines the mechanism by which a transaction input is authorized to be consumed.
#[derive(Clone, Debug, Eq, PartialEq, Hash, From, Packable)]
#[cfg_attr(
    feature = "serde",
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
    /// Returns the unlock kind of an [`UnlockBlock`].
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = Error, with = |e| e.unwrap_item_err_or_else(|p| Error::InvalidUnlockBlockCount(p.into())))]
pub struct UnlockBlocks(
    #[packable(verify_with = verify_unlock_blocks)] BoxedSlicePrefix<UnlockBlock, UnlockBlockCount>,
);

impl UnlockBlocks {
    /// Creates a new [`UnlockBlocks`].
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, Error> {
        let unlock_blocks: BoxedSlicePrefix<UnlockBlock, UnlockBlockCount> = unlock_blocks
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidUnlockBlockCount)?;

        verify_unlock_blocks::<true>(&unlock_blocks)?;

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

fn verify_unlock_blocks<const VERIFY: bool>(unlock_blocks: &[UnlockBlock]) -> Result<(), Error> {
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
                    || reference.index() >= index
                    || !matches!(unlock_blocks[reference.index() as usize], UnlockBlock::Signature(_))
                {
                    return Err(Error::InvalidUnlockBlockReference(index));
                }
            }
            UnlockBlock::Alias(alias) => {
                if index == 0 || alias.index() >= index {
                    return Err(Error::InvalidUnlockBlockAlias(index));
                }
            }
            UnlockBlock::Nft(nft) => {
                if index == 0 || nft.index() >= index {
                    return Err(Error::InvalidUnlockBlockNft(index));
                }
            }
        }
    }

    Ok(())
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    use super::*;
    pub use super::{
        alias::dto::AliasUnlockBlockDto, nft::dto::NftUnlockBlockDto, reference::dto::ReferenceUnlockBlockDto,
        signature::dto::SignatureUnlockBlockDto,
    };
    use crate::{
        error::dto::DtoError,
        signature::{
            dto::{Ed25519SignatureDto, SignatureDto},
            Ed25519Signature, Signature,
        },
    };

    /// Describes all the different unlock types.
    #[derive(Clone, Debug)]
    pub enum UnlockBlockDto {
        Signature(SignatureUnlockBlockDto),
        Reference(ReferenceUnlockBlockDto),
        Alias(AliasUnlockBlockDto),
        Nft(NftUnlockBlockDto),
    }

    impl From<&UnlockBlock> for UnlockBlockDto {
        fn from(value: &UnlockBlock) -> Self {
            match value {
                UnlockBlock::Signature(signature) => match signature.signature() {
                    Signature::Ed25519(ed) => UnlockBlockDto::Signature(SignatureUnlockBlockDto {
                        kind: SignatureUnlockBlock::KIND,
                        signature: SignatureDto::Ed25519(Ed25519SignatureDto {
                            kind: Ed25519Signature::KIND,
                            public_key: prefix_hex::encode(ed.public_key()),
                            signature: prefix_hex::encode(ed.signature()),
                        }),
                    }),
                },
                UnlockBlock::Reference(r) => UnlockBlockDto::Reference(ReferenceUnlockBlockDto {
                    kind: ReferenceUnlockBlock::KIND,
                    index: r.index(),
                }),
                UnlockBlock::Alias(a) => UnlockBlockDto::Alias(AliasUnlockBlockDto {
                    kind: AliasUnlockBlock::KIND,
                    index: a.index(),
                }),
                UnlockBlock::Nft(n) => UnlockBlockDto::Nft(NftUnlockBlockDto {
                    kind: NftUnlockBlock::KIND,
                    index: n.index(),
                }),
            }
        }
    }

    impl TryFrom<&UnlockBlockDto> for UnlockBlock {
        type Error = DtoError;

        fn try_from(value: &UnlockBlockDto) -> Result<Self, Self::Error> {
            match value {
                UnlockBlockDto::Signature(s) => match &s.signature {
                    SignatureDto::Ed25519(ed) => {
                        let public_key =
                            prefix_hex::decode(&ed.public_key).map_err(|_| DtoError::InvalidField("publicKey"))?;
                        let signature =
                            prefix_hex::decode(&ed.signature).map_err(|_| DtoError::InvalidField("signature"))?;
                        Ok(UnlockBlock::Signature(SignatureUnlockBlock::from(Signature::Ed25519(
                            Ed25519Signature::new(public_key, signature),
                        ))))
                    }
                },
                UnlockBlockDto::Reference(r) => Ok(UnlockBlock::Reference(ReferenceUnlockBlock::new(r.index)?)),
                UnlockBlockDto::Alias(a) => Ok(UnlockBlock::Alias(AliasUnlockBlock::new(a.index)?)),
                UnlockBlockDto::Nft(n) => Ok(UnlockBlock::Nft(NftUnlockBlock::new(n.index)?)),
            }
        }
    }

    impl<'de> Deserialize<'de> for UnlockBlockDto {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let value = Value::deserialize(d)?;
            Ok(
                match value
                    .get("type")
                    .and_then(Value::as_u64)
                    .ok_or_else(|| serde::de::Error::custom("invalid unlock block type"))? as u8
                {
                    SignatureUnlockBlock::KIND => {
                        UnlockBlockDto::Signature(SignatureUnlockBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize signature unlock block: {}", e))
                        })?)
                    }
                    ReferenceUnlockBlock::KIND => {
                        UnlockBlockDto::Reference(ReferenceUnlockBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize reference unlock block: {}", e))
                        })?)
                    }
                    AliasUnlockBlock::KIND => {
                        UnlockBlockDto::Alias(AliasUnlockBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize alias unlock block: {}", e))
                        })?)
                    }
                    NftUnlockBlock::KIND => {
                        UnlockBlockDto::Nft(NftUnlockBlockDto::deserialize(value).map_err(|e| {
                            serde::de::Error::custom(format!("cannot deserialize NFT unlock block: {}", e))
                        })?)
                    }
                    _ => return Err(serde::de::Error::custom("invalid unlock block type")),
                },
            )
        }
    }

    impl Serialize for UnlockBlockDto {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            #[derive(Serialize)]
            #[serde(untagged)]
            enum UnlockBlockDto_<'a> {
                T1(&'a SignatureUnlockBlockDto),
                T2(&'a ReferenceUnlockBlockDto),
                T3(&'a AliasUnlockBlockDto),
                T4(&'a NftUnlockBlockDto),
            }
            #[derive(Serialize)]
            struct TypedUnlockBlock<'a> {
                #[serde(flatten)]
                unlock_block: UnlockBlockDto_<'a>,
            }
            let unlock_block = match self {
                UnlockBlockDto::Signature(o) => TypedUnlockBlock {
                    unlock_block: UnlockBlockDto_::T1(o),
                },
                UnlockBlockDto::Reference(o) => TypedUnlockBlock {
                    unlock_block: UnlockBlockDto_::T2(o),
                },
                UnlockBlockDto::Alias(o) => TypedUnlockBlock {
                    unlock_block: UnlockBlockDto_::T3(o),
                },
                UnlockBlockDto::Nft(o) => TypedUnlockBlock {
                    unlock_block: UnlockBlockDto_::T4(o),
                },
            };
            unlock_block.serialize(serializer)
        }
    }
}
