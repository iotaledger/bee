// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;
mod signature;

pub use reference::ReferenceUnlock;
use reference::REFERENCE_UNLOCK_KIND;
use signature::SIGNATURE_UNLOCK_KIND;
pub use signature::{Ed25519Signature, SignatureUnlock};

use crate::{constants::UNLOCK_BLOCK_COUNT_RANGE, Error};

use bee_common::packable::{Packable, Read, Write};

use core::ops::Deref;

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum UnlockBlock {
    Signature(SignatureUnlock),
    Reference(ReferenceUnlock),
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
            Self::Signature(unlock) => SIGNATURE_UNLOCK_KIND.packed_len() + unlock.packed_len(),
            Self::Reference(unlock) => REFERENCE_UNLOCK_KIND.packed_len() + unlock.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Signature(unlock) => {
                SIGNATURE_UNLOCK_KIND.pack(writer)?;
                unlock.pack(writer)?;
            }
            Self::Reference(unlock) => {
                REFERENCE_UNLOCK_KIND.pack(writer)?;
                unlock.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            SIGNATURE_UNLOCK_KIND => SignatureUnlock::unpack(reader)?.into(),
            REFERENCE_UNLOCK_KIND => ReferenceUnlock::unpack(reader)?.into(),
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let unlock_blocks_len = u16::unpack(reader)? as usize;

        if !UNLOCK_BLOCK_COUNT_RANGE.contains(&unlock_blocks_len) {
            return Err(Error::InvalidUnlockBlockCount(unlock_blocks_len));
        }

        let mut unlock_blocks = Vec::with_capacity(unlock_blocks_len);
        for _ in 0..unlock_blocks_len {
            unlock_blocks.push(UnlockBlock::unpack(reader)?);
        }

        Ok(Self(unlock_blocks.into_boxed_slice()))
    }
}
