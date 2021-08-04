// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::{Ed25519Address, ED25519_ADDRESS_LENGTH};

use crate::{error::ValidationError, signature::SignatureUnlock};

use bee_packable::Packable;

use bech32::{self, FromBase32, ToBase32, Variant};

use alloc::{str::FromStr, string::String, vec::Vec};
use core::convert::TryFrom;

/// A generic address supporting different address kinds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8)]
pub enum Address {
    /// An Ed25519 address.
    #[packable(tag = 0)]
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
    pub fn try_from_bech32(addr: &str) -> Result<Self, ValidationError> {
        match bech32::decode(addr) {
            Ok((_hrp, data, _)) => {
                let bytes = Vec::<u8>::from_base32(&data).map_err(|_| ValidationError::InvalidAddress)?;
                Self::unpack_from_slice(bytes).map_err(|_| ValidationError::InvalidAddress)
            }
            Err(_) => Err(ValidationError::InvalidAddress),
        }
    }

    /// Encodes this address to a Bech32 string with the hrp (human readable part) argument as prefix.
    pub fn to_bech32(self, hrp: &str) -> String {
        let bytes = self.pack_to_vec().unwrap();

        bech32::encode(hrp, bytes.to_base32(), Variant::Bech32).expect("Invalid address.")
    }

    /// Verifies a [`SignatureUnlock`] for a message against the [`Address`].
    pub fn verify(&self, msg: &[u8], signature: &SignatureUnlock) -> Result<(), ValidationError> {
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
    type Err = ValidationError;

    fn from_str(address: &str) -> Result<Self, Self::Err> {
        Address::try_from_bech32(address)
    }
}

impl TryFrom<String> for Address {
    type Error = ValidationError;

    fn try_from(address: String) -> Result<Self, Self::Error> {
        Address::from_str(&address)
    }
}
