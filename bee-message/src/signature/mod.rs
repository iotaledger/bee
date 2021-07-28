// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::Ed25519Signature;

use crate::MessageUnpackError;

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use core::{convert::Infallible, fmt};

/// Error encountered unpacking a [`Signature`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureUnpackError {
    InvalidSignatureKind(u8),
}

impl fmt::Display for SignatureUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignatureKind(kind) => write!(f, "invalid Signature kind: {}", kind),
        }
    }
}

/// A signature used to validate the authenticity and integrity of a message.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum Signature {
    /// An Ed25519 signature.
    Ed25519(Ed25519Signature),
}

impl Signature {
    /// Returns the signature kind of a [`Signature`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Signature::KIND,
        }
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl Packable for Signature {
    type PackError = Infallible;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.kind().packed_len()
            + match self {
                Self::Ed25519(s) => s.packed_len(),
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.kind().pack(packer).map_err(PackError::infallible)?;

        match self {
            Self::Ed25519(s) => s.pack(packer).map_err(PackError::infallible)?,
        }

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let variant = match u8::unpack(unpacker).map_err(UnpackError::infallible)? {
            Ed25519Signature::KIND => {
                Self::Ed25519(Ed25519Signature::unpack(unpacker).map_err(UnpackError::infallible)?)
            }
            kind => {
                return Err(UnpackError::Packable(
                    SignatureUnpackError::InvalidSignatureKind(kind).into(),
                ));
            }
        };

        Ok(variant)
    }
}
