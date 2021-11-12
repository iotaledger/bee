// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the salt declaration payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    signature::Ed25519Signature,
    MessageUnpackError, ValidationError,
};

use bee_packable::{
    bounded::{BoundedU32, InvalidBoundedU32},
    prefix::{TryIntoPrefixError, UnpackPrefixError, VecPrefix},
    Packable,
};

use alloc::vec::Vec;
use core::convert::Infallible;

/// Maximum size of payload, minus prefix `u32` and timestamp `u64`.
pub(crate) const PREFIXED_SALT_BYTES_LENGTH_MAX: u32 = PAYLOAD_LENGTH_MAX - 12;

fn unpack_prefix_to_validation_error(
    err: UnpackPrefixError<Infallible, InvalidBoundedU32<0, PREFIXED_SALT_BYTES_LENGTH_MAX>>,
) -> ValidationError {
    ValidationError::InvalidSaltBytesLength(TryIntoPrefixError::Invalid(err.into_prefix()))
}

/// Represents a [`Salt`] used in a [`SaltDeclarationPayload`].
#[derive(Clone, Debug, PartialEq, Eq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct Salt {
    /// The value of the [`Salt`].
    #[packable(unpack_error_with = unpack_prefix_to_validation_error)]
    bytes: VecPrefix<u8, BoundedU32<0, PREFIXED_SALT_BYTES_LENGTH_MAX>>,
    /// The expiry time of the [`Salt`].
    expiry_time: u64,
}

impl Salt {
    /// Creates a new [`Salt`].
    pub fn new(bytes: Vec<u8>, expiry_time: u64) -> Result<Self, ValidationError> {
        Ok(Self {
            bytes: bytes.try_into().map_err(ValidationError::InvalidSaltBytesLength)?,
            expiry_time,
        })
    }

    /// Returns the value of the [`Salt`].
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the expiration time of the [`Salt`].
    pub fn expiry_time(&self) -> u64 {
        self.expiry_time
    }
}

/// A [`SaltDeclarationPayload`].
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct SaltDeclarationPayload {
    /// The declaring node ID (which may be different from the node ID of the message issuer).
    node_id: u32,
    /// The public salt of the requester.
    salt: Salt,
    /// The timestamp of the payload.
    timestamp: u64,
    /// The node signature.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    signature: [u8; Ed25519Signature::SIGNATURE_LENGTH],
}

impl MessagePayload for SaltDeclarationPayload {
    const KIND: u32 = 7;
    const VERSION: u8 = 0;
}

impl SaltDeclarationPayload {
    /// Creates a new [`SaltDeclarationPayloadBuilder`].
    pub fn builder() -> SaltDeclarationPayloadBuilder {
        SaltDeclarationPayloadBuilder::new()
    }

    /// Returns the node ID of a [`SaltDeclarationPayload`].
    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    /// Returns the salt of a [`SaltDeclarationPayload`].
    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    /// Returns the timestamp of a [`SaltDeclarationPayload`].
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the signature of a [`SaltDeclarationPayload`].
    pub fn signature(&self) -> &[u8; Ed25519Signature::SIGNATURE_LENGTH] {
        &self.signature
    }
}

/// A builder to build a [`SaltDeclarationPayload`].
#[derive(Default)]
pub struct SaltDeclarationPayloadBuilder {
    node_id: Option<u32>,
    salt: Option<Salt>,
    timestamp: Option<u64>,
    signature: Option<[u8; Ed25519Signature::SIGNATURE_LENGTH]>,
}

impl SaltDeclarationPayloadBuilder {
    /// Creates a new [`SaltDeclarationPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node ID to a [`SaltDeclarationPayloadBuilder`].
    pub fn with_node_id(mut self, node_id: u32) -> Self {
        self.node_id.replace(node_id);
        self
    }

    /// Adds a salt to a [`SaltDeclarationPayloadBuilder`].
    pub fn with_salt(mut self, salt: Salt) -> Self {
        self.salt.replace(salt);
        self
    }

    /// Adds a timestamp to a [`SaltDeclarationPayloadBuilder`].
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp.replace(timestamp);
        self
    }

    /// Adds a signature to a [`SaltDeclarationPayloadBuilder`].
    pub fn with_signature(mut self, signature: [u8; Ed25519Signature::SIGNATURE_LENGTH]) -> Self {
        self.signature.replace(signature);
        self
    }

    /// Consumes the [`SaltDeclarationPayloadBuilder`] and builds a [`SaltDeclarationPayload`].
    pub fn finish(self) -> Result<SaltDeclarationPayload, ValidationError> {
        let node_id = self.node_id.ok_or(ValidationError::MissingBuilderField("node_id"))?;
        let salt = self.salt.ok_or(ValidationError::MissingBuilderField("salt"))?;
        let timestamp = self
            .timestamp
            .ok_or(ValidationError::MissingBuilderField("timestamp"))?;
        let signature = self
            .signature
            .ok_or(ValidationError::MissingBuilderField("signature"))?;

        Ok(SaltDeclarationPayload {
            node_id,
            salt,
            timestamp,
            signature,
        })
    }
}
