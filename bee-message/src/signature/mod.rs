// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::{Ed25519Signature, ED25519_PUBLIC_KEY_LENGTH, ED25519_SIGNATURE_LENGTH};

use crate::MessageUnpackError;

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use core::{convert::Infallible, fmt};

/// Error encountered unpacking a `SignatureUnlock`.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SignatureUnlockUnpackError {
    InvalidSignatureUnlockKind(u8),
}

impl fmt::Display for SignatureUnlockUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignatureUnlockKind(kind) => write!(f, "invalid SignatureUnlock kind: {}", kind),
        }
    }
}

/// A `SignatureUnlock` contains a signature which is used to unlock a transaction input.
///
/// This is defined as part of the Unspent Transaction Output (UTXO) transaction protocol.
///
/// RFC: <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#signature-unlock-block>
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum SignatureUnlock {
    /// An Ed25519 signature.
    Ed25519(Ed25519Signature),
}

impl SignatureUnlock {
    /// The unlock kind of a `SignatureUnlock`
    pub const KIND: u8 = 0;

    /// Returns the signature kind of a `SignatureUnlock`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Signature::KIND,
        }
    }
}

impl From<Ed25519Signature> for SignatureUnlock {
    fn from(signature: Ed25519Signature) -> Self {
        Self::Ed25519(signature)
    }
}

impl Packable for SignatureUnlock {
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
                    SignatureUnlockUnpackError::InvalidSignatureUnlockKind(kind).into(),
                ));
            }
        };

        Ok(variant)
    }
}
