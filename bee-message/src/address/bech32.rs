// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{address::Address, error::ValidationError};

use bee_packable::Packable;

use bech32::{self, FromBase32, ToBase32};

use alloc::{
    boxed::Box,
    fmt,
    string::{String, ToString},
    vec::Vec,
};
use core::convert::{TryFrom, TryInto};

/// Wrapper for a `Bech32` encoded address string.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bech32Address(String);

impl Bech32Address {
    /// Creates a new [`Bech32Address`].
    pub fn new(s: &str) -> Result<Self, ValidationError> {
        match bech32::decode(s) {
            Ok((_, data, _)) => {
                let _ = Vec::<u8>::from_base32(&data).map_err(|_| ValidationError::InvalidAddress)?;
            }
            Err(_) => return Err(ValidationError::InvalidAddress),
        }

        Ok(Self(s.to_string()))
    }

    /// Returns the HRP part of a [`Bech32Address`].
    pub fn hrp(&self) -> String {
        // Unwrap is fine since self is valid.
        let (hrp, _, _) = bech32::decode(&self.0).unwrap();

        hrp
    }

    /// Returns the data part of a [`Bech32Address`].
    pub fn data(&self) -> Box<[u8]> {
        // Unwrap is fine since self is valid.
        let (_, data, _) = bech32::decode(&self.0).unwrap();

        // Unwrap is fine since self is valid.
        Vec::<u8>::from_base32(&data).unwrap().into_boxed_slice()
    }

    /// Creates a wrapped [`Bech32Address`] from an [`Address`] and a human-readable part.
    pub fn from_address(hrp: &str, address: &Address) -> Self {
        // Unwrap is fine since packing to vec can't fail.
        let bytes = address.pack_to_vec().unwrap();

        // Unwrap is fine since we know the [`Address`] is valid.
        Self(bech32::encode(hrp, bytes.to_base32(), bech32::Variant::Bech32).unwrap())
    }
}

impl TryInto<Address> for Bech32Address {
    type Error = ValidationError;

    fn try_into(self) -> Result<Address, Self::Error> {
        Address::unpack_from_slice(self.data()).map_err(|_| ValidationError::InvalidAddress)
    }
}

impl TryFrom<&str> for Bech32Address {
    type Error = ValidationError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<String> for Bech32Address {
    type Error = ValidationError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(&s)
    }
}

impl fmt::Display for Bech32Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
