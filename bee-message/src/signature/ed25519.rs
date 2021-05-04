// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::boxed::Box;
use core::convert::{Infallible, TryInto};

/// Length (in bytes) of an Ed25519 public key.
pub const ED25519_PUBLIC_KEY_LENGTH: usize = 32;

/// Length (in bytes) of an Ed26618 signature.
pub const ED25519_SIGNATURE_LENGTH: usize = 64;

/// An Ed25519 signature.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
            signature: signature.into(),
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
    type PackError = Infallible;
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        ED25519_PUBLIC_KEY_LENGTH + ED25519_SIGNATURE_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.public_key.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.signature` is known to be 64 bytes.
        let signature_bytes: [u8; ED25519_SIGNATURE_LENGTH] = self.signature.to_vec().try_into().unwrap();
        signature_bytes.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let public_key = <[u8; ED25519_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;
        let signature = <[u8; ED25519_SIGNATURE_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();

        Ok(Self { public_key, signature })
    }
}
