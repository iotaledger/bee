// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;
mod wots;

pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};
pub use wots::WotsAddress;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::string::String;

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Address {
    Wots(WotsAddress),
    Ed25519(Ed25519Address),
}

impl From<WotsAddress> for Address {
    fn from(address: WotsAddress) -> Self {
        Self::Wots(address)
    }
}

impl From<Ed25519Address> for Address {
    fn from(address: Ed25519Address) -> Self {
        Self::Ed25519(address)
    }
}

impl AsRef<[u8]> for Address {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Wots(address) => address.as_ref(),
            Self::Ed25519(address) => address.as_ref(),
        }
    }
}

impl Address {
    pub fn to_bech32(&self) -> String {
        match self {
            Address::Wots(address) => address.to_bech32(),
            Address::Ed25519(address) => address.to_bech32(),
        }
    }
}

impl Packable for Address {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Wots(address) => 0u8.packed_len() + address.packed_len(),
            Self::Ed25519(address) => 1u8.packed_len() + address.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Wots(address) => {
                0u8.pack(writer)?;
                address.pack(writer)?;
            }
            Self::Ed25519(address) => {
                1u8.pack(writer)?;
                address.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(reader)? {
            0 => Self::Wots(WotsAddress::unpack(reader)?),
            1 => Self::Ed25519(Ed25519Address::unpack(reader)?),
            _ => return Err(Self::Error::InvalidVariant),
        })
    }
}
