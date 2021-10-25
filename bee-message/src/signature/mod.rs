// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::Ed25519Signature;

use crate::Error;

use bee_packable::Packable;

/// A `SignatureUnlock` contains a signature which is used to unlock a transaction input.
///
/// This is defined as part of the Unspent Transaction Output (UTXO) transaction protocol.
///
/// RFC: <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#signature-unlock-block>
#[derive(Clone, Debug, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidSignatureKind)]
#[packable(unpack_error = Error)]
pub enum SignatureUnlock {
    /// An Ed25519 signature.
    #[packable(tag = Ed25519Signature::KIND)]
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
