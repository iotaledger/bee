// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{signature::Ed25519Signature, Error};

use bee_common::packable::{Packable, Read, Write};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature},
};

use core::{convert::TryInto, str::FromStr};

/// The number of bytes in an Ed25519 address.
/// See <https://en.wikipedia.org/wiki/EdDSA#Ed25519> for more information.
pub const ED25519_ADDRESS_LENGTH: usize = 32;

/// An Ed25519 address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ed25519Address([u8; ED25519_ADDRESS_LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl Ed25519Address {
    /// The address kind of an Ed25519 address.
    pub const KIND: u8 = 0;

    /// Creates a new Ed25519 address.
    pub fn new(address: [u8; ED25519_ADDRESS_LENGTH]) -> Self {
        address.into()
    }

    /// Returns the length of an Ed25519 address.
    pub fn len(&self) -> usize {
        ED25519_ADDRESS_LENGTH
    }

    /// Verifies a [`Ed25519Signature`] for a message against the [`Ed25519Address`].
    pub fn verify(&self, msg: &[u8], signature: &Ed25519Signature) -> Result<(), Error> {
        let address = Blake2b256::digest(signature.public_key());

        if self.0 != *address {
            return Err(Error::SignaturePublicKeyMismatch(
                hex::encode(self.0),
                hex::encode(address),
            ));
        }

        if !PublicKey::try_from_bytes(*signature.public_key())?
            // This unwrap is fine as the length of the signature has already been verified at construction.
            .verify(&Signature::from_bytes(signature.signature().try_into().unwrap()), msg)
        {
            return Err(Error::InvalidSignature);
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
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
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(ED25519_ADDRESS_LENGTH * 2, s.len()))?;

        Ok(Ed25519Address::from(bytes))
    }
}

impl AsRef<[u8]> for Ed25519Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
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
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(<[u8; ED25519_ADDRESS_LENGTH]>::unpack_inner::<R, CHECK>(
            reader,
        )?))
    }
}
