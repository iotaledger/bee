// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::convert::{TryFrom, TryInto};

use crate::error::ValidationError;
use super::Address;

use bee_packable::Packable;

use bech32::{self, FromBase32, ToBase32};

use alloc::{
    boxed::Box,
    fmt,
    string::{String, ToString},
    vec::Vec,
};

/// Wrapper for a `Bech32` encoded string.
#[derive(Debug)]
pub struct Bech32Address(String);

impl Bech32Address {
    /// Returns the data part of a [`Bech32Address`].
    pub fn data(&self) -> Box<[u8]> {
        let (_hrp, data, _) = bech32::decode(&self.0).unwrap();

        Vec::<u8>::from_base32(&data).unwrap().into_boxed_slice()
    }

    /// Returns the hrp part of a [`Bech32Address`].
    pub fn hrp(&self) -> String {
        let (hrp, _data, _) = bech32::decode(&self.0).unwrap();

        hrp
    }

    /// Creates a wrapped [`Bech32Address`] from an [`Address`] and a human-readable portion.
    pub fn from_address(hrp: &str, address: &Address) -> Self {
        let bytes = address.pack_to_vec().unwrap();

        // Unwrap is fine here, since we know the [`Address`] is valid.
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
        let _ = bech32::decode(s).map_err(|_| ValidationError::InvalidAddress)?;

        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for Bech32Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
