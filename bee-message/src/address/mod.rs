// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module that provides types and syntactic validations of addresses.

mod bech32;
mod bls;
mod ed25519;

pub use self::bech32::Bech32Address;
pub use bls::BlsAddress;
pub use ed25519::Ed25519Address;

use crate::{
    error::{MessageUnpackError, ValidationError},
    signature::Signature,
};

use bee_packable::Packable;

use core::fmt;

/// Error encountered unpacking an [`Address`].
#[derive(Debug)]
#[allow(missing_docs)]
pub enum AddressUnpackError {
    InvalidKind(u8),
}

impl fmt::Display for AddressUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKind(kind) => write!(f, "invalid Address kind: {}", kind),
        }
    }
}

/// A generic address supporting different address kinds.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = AddressUnpackError::InvalidKind)]
#[packable(unpack_error = MessageUnpackError)]
pub enum Address {
    /// An Ed25519 address.
    #[packable(tag = Ed25519Address::KIND)]
    Ed25519(Ed25519Address),
    /// A BLS address.
    #[packable(tag = BlsAddress::KIND)]
    Bls(BlsAddress),
}

impl_wrapped_variant!(Address, Address::Ed25519, Ed25519Address);
impl_wrapped_variant!(Address, Address::Bls, BlsAddress);

impl Address {
    /// Returns the address kind of an [`Address`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Ed25519(_) => Ed25519Address::KIND,
            Self::Bls(_) => BlsAddress::KIND,
        }
    }

    /// Returns the length, in bytes, of an [`Address`], depending on the kind.
    pub fn length(&self) -> usize {
        match self {
            Self::Ed25519(_) => Ed25519Address::LENGTH,
            Self::Bls(_) => BlsAddress::LENGTH,
        }
    }

    /// Verifies a [`Signature`] for a message against the [`Address`].
    pub fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), ValidationError> {
        match self {
            Address::Ed25519(address) => {
                if let Signature::Ed25519(signature) = signature {
                    address.verify(signature, msg)
                } else {
                    Err(ValidationError::AddressSignatureKindMismatch {
                        expected: self.kind(),
                        actual: signature.kind(),
                    })
                }
            }
            Address::Bls(_) => {
                if let Signature::Bls(_) = signature {
                    // TODO BLS address verification
                    Err(ValidationError::InvalidAddressKind(BlsAddress::KIND))
                } else {
                    Err(ValidationError::AddressSignatureKindMismatch {
                        expected: self.kind(),
                        actual: signature.kind(),
                    })
                }
            }
        }
    }
}

impl core::fmt::Display for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // Encodes to a base 16 hexadecimal string.
        match self {
            Self::Ed25519(address) => write!(f, "{}", hex::encode(address)),
            Self::Bls(address) => write!(f, "{}", hex::encode(address)),
        }
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::Ed25519(address) => write!(f, "{:?}", address),
            Self::Bls(address) => write!(f, "{:?}", address),
        }
    }
}
