// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod ed25519;

pub use ed25519::Ed25519Signature;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

/// A `SignatureUnlock` contains a signature which is used to unlock a transaction input.
///
/// This is defined as part of the Unspent Transaction Output (UTXO) transaction protocol. The other
/// signature type is a [ReferenceUnlock](crate::unlock::ReferenceUnlock) which refers to a previous
/// `SignatureUnlock` in the [UnlockBlocks](crate::unlock::UnlockBlocks) list when the same
/// signature can be used to unlock multiple inputs.
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#signature-unlock-block>
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
pub enum SignatureUnlock {
    /// An Ed25519 signature which unlocks a transaction input.
    Ed25519(Ed25519Signature),
}

impl SignatureUnlock {
    /// The kind of unlock block used to unlock an input: `0` as defined by the protocol.
    pub const KIND: u8 = 0;

    /// The kind of the signature used to unlock a transaction input. Defined by the underlying
    /// signature type.
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
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Ed25519(signature) => Ed25519Signature::KIND.packed_len() + signature.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Ed25519(signature) => {
                Ed25519Signature::KIND.pack(writer)?;
                signature.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(match u8::unpack_inner::<R, CHECK>(reader)? {
            Ed25519Signature::KIND => Ed25519Signature::unpack_inner::<R, CHECK>(reader)?.into(),
            k => return Err(Self::Error::InvalidSignatureKind(k)),
        })
    }
}
