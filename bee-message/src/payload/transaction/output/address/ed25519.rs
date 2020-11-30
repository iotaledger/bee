// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use bech32::{self, ToBase32};

use alloc::{string::String, vec};
use core::{convert::TryInto, str::FromStr};

pub const ED25519_ADDRESS_LENGTH: usize = 32;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ed25519Address([u8; ED25519_ADDRESS_LENGTH]);

string_serde_impl!(Ed25519Address);

impl From<[u8; ED25519_ADDRESS_LENGTH]> for Ed25519Address {
    fn from(bytes: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for Ed25519Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; ED25519_ADDRESS_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHex)?
            .as_slice()
            .try_into()
            .map_err(|_| Self::Err::InvalidHex)?;

        Ok(Ed25519Address::from(bytes))
    }
}

impl AsRef<[u8]> for Ed25519Address {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Ed25519Address {
    pub fn new(address: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    pub fn len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn to_bech32(&self) -> String {
        let mut serialized = vec![1u8];
        serialized.extend_from_slice(&self.0);
        bech32::encode("iot", serialized.to_base32()).expect("Valid Ed25519 address required.")
    }
}

impl core::fmt::Display for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for Ed25519Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Ed25519Address({})", self)
    }
}

impl Packable for Ed25519Address {
    type Error = Error;

    fn packed_len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; ED25519_ADDRESS_LENGTH];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
