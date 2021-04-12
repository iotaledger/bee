// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;

pub use reference::ReferenceUnlock;

use crate::{constants::UNLOCK_BLOCK_COUNT_RANGE, signature::SignatureUnlock, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::Deref;
// TODO no_std
use std::collections::HashSet;

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum UnlockBlock {
    Signature(SignatureUnlock),
    Reference(ReferenceUnlock),
}

impl UnlockBlock {
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

impl Packable for UnlockBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Signature(unlock) => SignatureUnlock::KIND.packed_len() + unlock.packed_len(),
            Self::Reference(unlock) => ReferenceUnlock::KIND.packed_len() + unlock.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Signature(unlock) => {
                SignatureUnlock::KIND.pack(writer)?;
                unlock.pack(writer)?;
            }
            Self::Reference(unlock) => {
                ReferenceUnlock::KIND.pack(writer)?;
                unlock.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            SignatureUnlock::KIND => SignatureUnlock::unpack_inner::<R, CHECK>(reader)?.into(),
            ReferenceUnlock::KIND => ReferenceUnlock::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidUnlockBlockKind(k)),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnlockBlocks(Box<[UnlockBlock]>);

impl UnlockBlocks {
    pub fn new(unlock_blocks: Vec<UnlockBlock>) -> Result<Self, Error> {
        if !UNLOCK_BLOCK_COUNT_RANGE.contains(&unlock_blocks.len()) {
            return Err(Error::InvalidUnlockBlockCount(unlock_blocks.len()));
        }

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

        Ok(Self(unlock_blocks.into_boxed_slice()))
    }

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
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u16.packed_len() + self.0.iter().map(Packable::packed_len).sum::<usize>()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.0.len() as u16).pack(writer)?;
        for unlock_block in self.0.as_ref() {
            unlock_block.pack(writer)?;
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let unlock_blocks_len = u16::unpack_inner::<R, CHECK>(reader)? as usize;

        if CHECK && !UNLOCK_BLOCK_COUNT_RANGE.contains(&unlock_blocks_len) {
            return Err(Error::InvalidUnlockBlockCount(unlock_blocks_len));
        }

        let mut unlock_blocks = Vec::with_capacity(unlock_blocks_len);
        for _ in 0..unlock_blocks_len {
            unlock_blocks.push(UnlockBlock::unpack_inner::<R, CHECK>(reader)?);
        }

        Self::new(unlock_blocks)
    }
}
