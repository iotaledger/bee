// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};

use crate::{signature::SignatureUnlock, Error};

use bee_common::packable::{Packable, Read, Write};

use bech32::{self, FromBase32, ToBase32, Variant};

use alloc::{str::FromStr, string::String};

/// A generic address supporting different address kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Address {
    /// An Ed25519 address.
    Ed25519(Ed25519Address),
}

impl Address {
    /// Returns the address kind of an `Address`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Address::KIND,
        }
    }

    /// Tries to create an `Address` from a Bech32 encoded string.
    pub fn try_from_bech32(addr: &str) -> Result<Self, Error> {
        match bech32::decode(addr) {
            Ok((_hrp, data, _)) => {
                let bytes = Vec::<u8>::from_base32(&data).map_err(|_| Error::InvalidAddress)?;
                Self::unpack(&mut bytes.as_slice()).map_err(|_| Error::InvalidAddress)
            }
            Err(_) => Err(Error::InvalidAddress),
        }
    }

    /// Encodes this address to a Bech32 string with the hrp (human readable part) argument as prefix.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_bech32(&self, hrp: &str) -> String {
        bech32::encode(hrp, self.pack_new().to_base32(), Variant::Bech32).expect("Invalid address.")
    }

    /// Verifies a [`SignatureUnlock`] for a message against the [`Address`].
    pub fn verify(&self, msg: &[u8], signature: &SignatureUnlock) -> Result<(), Error> {
        match self {
            Address::Ed25519(address) => {
                let SignatureUnlock::Ed25519(signature) = signature;
                address.verify(msg, signature)
            }
        }
    }
}

impl From<Ed25519Address> for Address {
    fn from(address: Ed25519Address) -> Self {
        Self::Ed25519(address)
    }
}

impl FromStr for Address {
    type Err = Error;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        Address::try_from_bech32(address)
    }
}

impl TryFrom<String> for Address {
    type Error = Error;

    fn try_from(address: String) -> Result<Self, Self::Error> {
        Address::from_str(&address)
    }
}

impl Packable for Address {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Ed25519(address) => Ed25519Address::KIND.packed_len() + address.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Ed25519(address) => {
                Ed25519Address::KIND.pack(writer)?;
                address.pack(writer)?;
            }
        }
        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            Ed25519Address::KIND => Ed25519Address::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidAddressKind(k)),
        })
    }
}
