// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

use ed25519::ED25519_ADDRESS_TYPE;
pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};

use crate::Error;

use bech32::FromBase32;
use bee_common::packable::{Packable, Read, Write};

use alloc::string::String;
use core::ops::Deref;
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
#[serde(tag = "type", content = "data")]
pub enum Address {
    Ed25519(Ed25519Address),
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
            Self::Ed25519(address) => address.as_ref(),
        }
    }
}

impl Address {
    pub fn try_from_bech32(addr: &str) -> Result<Self, Error> {
        match bech32::decode(&addr) {
            Ok((hrp, data)) => {
                if hrp.eq("iota") || hrp.eq("atoi") {
                    let bytes = Vec::<u8>::from_base32(&data).map_err(|_| Error::InvalidAddress)?;
                    Ok(Self::unpack(&mut bytes.as_slice()).map_err(|_| Error::InvalidAddress)?)
                } else {
                    Err(Error::InvalidAddress)
                }
            }
            Err(_) => Err(Error::InvalidAddress),
        }
    }
    pub fn to_bech32(&self) -> String {
        match self {
            Address::Ed25519(address) => address.to_bech32(),
        }
    }
    pub fn to_bech32_testnet(&self) -> String {
        match self {
            Address::Ed25519(address) => address.to_bech32_testnet(),
        }
    }
}

impl Packable for Address {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Ed25519(address) => ED25519_ADDRESS_TYPE.packed_len() + address.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Ed25519(address) => {
                ED25519_ADDRESS_TYPE.pack(writer)?;
                address.pack(writer)?;
            }
        }
        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack(reader)? {
            ED25519_ADDRESS_TYPE => Self::Ed25519(Ed25519Address::unpack(reader)?),
            _ => return Err(Self::Error::InvalidAddressType),
        })
    }
}

/// Bech32 encoded address struct
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
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
