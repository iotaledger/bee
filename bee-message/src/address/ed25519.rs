// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use crypto::signatures::ed25519::PUBLIC_KEY_LENGTH;
use derive_more::{AsRef, Deref, From};

use crate::Error;

/// An Ed25519 address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, From, AsRef, Deref, packable::Packable)]
#[as_ref(forward)]
pub struct Ed25519Address([u8; Self::LENGTH]);

impl Ed25519Address {
    /// The [`Address`](crate::address::Address) kind of an [`Ed25519Address`].
    pub const KIND: u8 = 0;
    /// The length of an [`Ed25519Address`].
    pub const LENGTH: usize = PUBLIC_KEY_LENGTH;

    /// Creates a new [`Ed25519Address`].
    #[inline(always)]
    pub fn new(address: [u8; Self::LENGTH]) -> Self {
        Self::from(address)
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(Ed25519Address);

impl FromStr for Ed25519Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ed25519Address::new(prefix_hex::decode(s).map_err(Error::HexError)?))
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.0))
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self)
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::error::dto::DtoError;

    /// Describes an Ed25519 address.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Ed25519AddressDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "pubKeyHash")]
        pub pub_key_hash: String,
    }

    impl From<&Ed25519Address> for Ed25519AddressDto {
        fn from(value: &Ed25519Address) -> Self {
            Self {
                kind: Ed25519Address::KIND,
                pub_key_hash: value.to_string(),
            }
        }
    }

    impl TryFrom<&Ed25519AddressDto> for Ed25519Address {
        type Error = DtoError;

        fn try_from(value: &Ed25519AddressDto) -> Result<Self, Self::Error> {
            value
                .pub_key_hash
                .parse::<Ed25519Address>()
                .map_err(|_| DtoError::InvalidField("Ed25519 address"))
        }
    }
}
