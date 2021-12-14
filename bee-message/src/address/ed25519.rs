// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{signature::Ed25519Signature, Error};

use bee_common::packable::{Packable, Read, Write};

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::{PublicKey, Signature},
};

use core::str::FromStr;

/// An Ed25519 address.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, derive_more::From, derive_more::AsRef)]
#[as_ref(forward)]
pub struct Ed25519Address([u8; Self::LENGTH]);

#[allow(clippy::len_without_is_empty)]
impl Ed25519Address {
    /// The [`Address`](crate::address::Address) kind of an [`Ed25519Signature`].
    pub const KIND: u8 = 0;
    /// The length of an [`Ed25519Signature`].
    pub const LENGTH: usize = 32;

    /// Creates a new Ed25519 address.
    #[inline(always)]
    pub fn new(address: [u8; Self::LENGTH]) -> Self {
        Self::from(address)
    }

    /// Verifies a [`Ed25519Signature`] for a message against the [`Ed25519Address`].
    pub fn verify(&self, msg: &[u8], signature: &Ed25519Signature) -> Result<(), Error> {
        let address = Blake2b256::digest(signature.public_key());

        if self.0 != *address {
            return Err(Error::SignaturePublicKeyMismatch {
                expected: hex::encode(self.0),
                actual: hex::encode(address),
            });
        }

        if !PublicKey::try_from_bytes(*signature.public_key())?
            // This unwrap is fine as the length of the signature has already been verified at construction.
            .verify(&Signature::from_bytes(*signature.signature()), msg)
        {
            return Err(Error::InvalidSignature);
        }

        Ok(())
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(Ed25519Address);

impl FromStr for Ed25519Address {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; Self::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength {
                expected: Self::LENGTH * 2,
                actual: s.len(),
            })?;

        Ok(Ed25519Address::from(bytes))
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
        Self::LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(<[u8; Self::LENGTH]>::unpack_inner::<R, CHECK>(reader)?))
    }
}
