// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod reference;
mod signature;

pub use reference::ReferenceUnlock;
pub use signature::{Ed25519Signature, SignatureUnlock, WotsSignature};

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "data")]
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
            Self::Signature(unlock) => 0u8.packed_len() + unlock.packed_len(),
            Self::Reference(unlock) => 1u8.packed_len() + unlock.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Signature(unlock) => {
                0u8.pack(writer)?;
                unlock.pack(writer)?;
            }
            Self::Reference(unlock) => {
                1u8.pack(writer)?;
                unlock.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            0 => Self::Signature(SignatureUnlock::unpack(reader)?),
            1 => Self::Reference(ReferenceUnlock::unpack(reader)?),
            _ => return Err(Self::Error::InvalidVariant),
        })
    }
}
