// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

pub use regular::{RegularTransactionEssence, RegularTransactionEssenceBuilder};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// A generic essence that can represent different types defining transaction essences.
#[derive(Clone, Debug, Eq, PartialEq, derive_more::From)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum TransactionEssence {
    /// A regular transaction essence.
    Regular(RegularTransactionEssence),
}

impl TransactionEssence {
    /// Returns the essence kind of an [`TransactionEssence`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => RegularTransactionEssence::KIND,
        }
    }

    /// Return the Blake2b hash of an [`TransactionEssence`].
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_new()).into()
    }
}

impl Packable for TransactionEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Regular(essence) => RegularTransactionEssence::KIND.packed_len() + essence.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Regular(essence) => {
                RegularTransactionEssence::KIND.pack(writer)?;
                essence.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            RegularTransactionEssence::KIND => RegularTransactionEssence::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidEssenceKind(k)),
        })
    }
}
