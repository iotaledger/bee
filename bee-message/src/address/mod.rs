// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};

use crate::{unlock::SignatureUnlock, Error};

use bee_common::packable::{Packable, Read, Write};

use bech32::FromBase32;

use alloc::string::String;
use core::ops::Deref;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Address {
    Ed25519(Ed25519Address),
}

impl Address {
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Address::KIND,
        }
    }

    pub fn try_from_bech32(addr: &str) -> Result<Self, Error> {
        match bech32::decode(&addr) {
            Ok((_hrp, data, _)) => {
                let bytes = Vec::<u8>::from_base32(&data).map_err(|_| Error::InvalidAddress)?;
                Self::unpack(&mut bytes.as_slice()).map_err(|_| Error::InvalidAddress)
            }
            Err(_) => Err(Error::InvalidAddress),
        }
    }

    pub fn to_bech32(&self, hrp: &str) -> String {
        match self {
            Address::Ed25519(address) => address.to_bech32(hrp),
        }
    }

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            Ed25519Address::KIND => Ed25519Address::unpack(reader)?.into(),
            k => return Err(Self::Error::InvalidAddressKind(k)),
        })
    }
}

/// Bech32 encoded address struct
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bech32Address(pub String);

impl Deref for Bech32Address {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for Bech32Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Bech32Address {
    fn from(address: String) -> Self {
        Bech32Address(address)
    }
}

impl From<&str> for Bech32Address {
    fn from(address: &str) -> Self {
        Bech32Address(address.to_string())
    }
}
