// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use alloc::boxed::Box;

const ED25519_PUBLIC_KEY_LENGTH: usize = 32;
const ED25519_SIGNATURE_LENGTH: usize = 64;

/// An Ed25519 signature.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Ed25519Signature {
    public_key: [u8; ED25519_PUBLIC_KEY_LENGTH],
    signature: Box<[u8]>,
}

impl Ed25519Signature {
    /// The signature kind of an `Ed25519Signature`.
    pub const KIND: u8 = 0;

    /// Creates a new `Ed25519Signature`.
    pub fn new(public_key: [u8; ED25519_PUBLIC_KEY_LENGTH], signature: [u8; ED25519_SIGNATURE_LENGTH]) -> Self {
        Self {
            public_key,
            signature: Box::new(signature),
        }
    }

    /// Returns the public key of an `Ed25519Signature`.
    pub fn public_key(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.public_key
    }

    /// Return the actual signature of an `Ed25519Signature`.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl Packable for Ed25519Signature {
    type Error = Error;

    fn packed_len(&self) -> usize {
        ED25519_PUBLIC_KEY_LENGTH + ED25519_SIGNATURE_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.public_key.pack(writer)?;
        writer.write_all(&self.signature)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let public_key = <[u8; ED25519_PUBLIC_KEY_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;
        let signature = <[u8; ED25519_SIGNATURE_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self::new(public_key, signature))
    }
}
