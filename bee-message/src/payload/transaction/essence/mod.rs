// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

pub use regular::{RegularEssence, RegularEssenceBuilder};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use crypto::hashes::{blake2b::Blake2b256, Digest};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Essence {
    Regular(RegularEssence),
}

impl Essence {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => RegularEssence::KIND,
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_new()).into()
    }
}

impl From<RegularEssence> for Essence {
    fn from(essence: RegularEssence) -> Self {
        Self::Regular(essence)
    }
}

impl Packable for Essence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Regular(essence) => RegularEssence::KIND.packed_len() + essence.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Regular(essence) => {
                RegularEssence::KIND.pack(writer)?;
                essence.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            RegularEssence::KIND => RegularEssence::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidEssenceKind(k)),
        })
    }
}
