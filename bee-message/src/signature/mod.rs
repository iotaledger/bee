// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of signatures.

mod bls;
mod ed25519;

pub use bls::BlsSignature;
pub use ed25519::Ed25519Signature;

use crate::MessageUnpackError;

use bee_packable::Packable;

use core::{convert::Infallible, fmt};

/// Error encountered unpacking a [`Signature`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureUnpackError {
    InvalidKind(u8),
}

impl fmt::Display for SignatureUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid Signature kind: {}", kind),
        }
    }
}

/// A signature used to validate the authenticity and integrity of a message.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8)]
#[packable(tag_error = SignatureUnpackError::InvalidSignatureKind)]
#[packable(pack_error = Infallible)]
#[packable(unpack_error = MessageUnpackError)]
pub enum Signature {
    /// An Ed25519 signature.
    #[packable(tag = Ed25519Signature::KIND)]
    Ed25519(Ed25519Signature),
    /// A BLS signature.
    #[packable(tag = BlsSignature::KIND)]
    Bls(BlsSignature),
}

impl Signature {
    /// Returns the signature kind of a [`Signature`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Signature::KIND,
            Self::Bls(_) => BlsSignature::KIND,
        }
    }
}

impl From<Ed25519Signature> for Signature {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl From<BlsSignature> for Signature {
    fn from(signature: BlsSignature) -> Self {
        Self::Bls(signature)
    }
}
