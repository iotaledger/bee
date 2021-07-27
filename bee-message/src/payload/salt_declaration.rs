// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module describing the salt declaration payload.

use crate::{
    payload::{MessagePayload, PAYLOAD_LENGTH_MAX},
    signature::ED25519_PUBLIC_KEY_LENGTH,
    MessagePackError, MessageUnpackError, ValidationError,
};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    PackError, Packable, Packer, UnpackError, Unpacker, VecPrefix,
};

use alloc::vec::Vec;
use core::{
    convert::{Infallible, TryInto},
    fmt,
};

/// Maximum size of payload, minus prefix `u32` and timestamp `u64`.
const PREFIXED_BYTES_LENGTH_MAX: usize = PAYLOAD_LENGTH_MAX - 12;

/// Error encountered packing a salt declaration payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SaltDeclarationPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u32>> for SaltDeclarationPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for SaltDeclarationPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for salt bytes"),
        }
    }
}

/// Error encountered unpacking a salt declaration payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum SaltDeclarationUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
}

impl_from_infallible!(SaltDeclarationUnpackError);

impl From<UnpackPrefixError<Infallible, u32>> for SaltDeclarationUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for SaltDeclarationUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for salt bytes"),
            Self::InvalidPrefixLength(len) => write!(f, "unpacked prefix larger than maximum specified: {}", len),
        }
    }
}

/// Represents a [`Salt`] used in a [`SaltDeclarationPayload`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Salt {
    /// The value of the [`Salt`].
    bytes: Vec<u8>,
    /// The expiration time of the [`Salt`].
    expiry_time: u64,
}

impl Salt {
    /// Creates a new [`Salt`].
    pub fn new(bytes: Vec<u8>, expiry_time: u64) -> Result<Self, ValidationError> {
        validate_bytes_length(bytes.len())?;

        Ok(Self { bytes, expiry_time })
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

impl Packable for Salt {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwrap is safe, since bytes length has already been validated.
        let prefixed_bytes: VecPrefix<u8, u32, PREFIXED_BYTES_LENGTH_MAX> = self.bytes.clone().try_into().unwrap();

        prefixed_bytes.packed_len() + self.expiry_time.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // Unwrap is safe, since bytes length has already been validated.
        let prefixed_bytes: VecPrefix<u8, u32, PREFIXED_BYTES_LENGTH_MAX> = self.bytes.clone().try_into().unwrap();
        prefixed_bytes
            .pack(packer)
            .map_err(PackError::coerce::<SaltDeclarationPackError>)
            .map_err(PackError::coerce)?;

        self.expiry_time.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let bytes: Vec<u8> = VecPrefix::<u8, u32, PREFIXED_BYTES_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<SaltDeclarationUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        validate_bytes_length(bytes.len()).map_err(|e| UnpackError::Packable(e.into()))?;

        let expiry_time = u64::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self { bytes, expiry_time })
    }
}

fn validate_bytes_length(len: usize) -> Result<(), ValidationError> {
    if len > PREFIXED_BYTES_LENGTH_MAX {
        Err(ValidationError::InvalidSaltDeclarationBytesLength(len))
    } else {
        Ok(())
    }
}

/// A [`SaltDeclarationPayload`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct SaltDeclarationPayload {
    /// The declaring node ID (which may be different from the node ID of the message issuer).
    node_id: u32,
    /// The public salt of the requester.
    salt: Salt,
    /// The timestamp of the payload.
    timestamp: u64,
    /// The node signature.
    signature: [u8; ED25519_PUBLIC_KEY_LENGTH],
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
    pub fn signature(&self) -> &[u8; ED25519_PUBLIC_KEY_LENGTH] {
        &self.signature
    }
}

impl Packable for SaltDeclarationPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        Self::VERSION.packed_len()
            + self.node_id.packed_len()
            + self.salt.packed_len()
            + self.timestamp.packed_len()
            + self.signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        Self::VERSION.pack(packer).map_err(PackError::infallible)?;
        self.node_id.pack(packer).map_err(PackError::infallible)?;
        self.salt.pack(packer).map_err(PackError::coerce)?;
        self.timestamp.pack(packer).map_err(PackError::infallible)?;
        self.signature.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_payload_version(version).map_err(|e| UnpackError::Packable(e.into()))?;

        let node_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let salt = Salt::unpack(unpacker).map_err(UnpackError::coerce)?;
        let timestamp = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let signature = <[u8; ED25519_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self {
            node_id,
            salt,
            timestamp,
            signature,
        })
    }
}

fn validate_payload_version(version: u8) -> Result<(), ValidationError> {
    if version != SaltDeclarationPayload::VERSION {
        Err(ValidationError::InvalidPayloadVersion {
            version,
            payload_kind: SaltDeclarationPayload::KIND,
        })
    } else {
        Ok(())
    }
}

/// A builder to build a [`SaltDeclarationPayload`].
#[derive(Default)]
pub struct SaltDeclarationPayloadBuilder {
    node_id: Option<u32>,
    salt: Option<Salt>,
    timestamp: Option<u64>,
    signature: Option<[u8; ED25519_PUBLIC_KEY_LENGTH]>,
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
    pub fn with_signature(mut self, signature: [u8; ED25519_PUBLIC_KEY_LENGTH]) -> Self {
        self.signature.replace(signature);
        self
    }

    /// Consumes the [`SaltDeclarationPayloadBuilder`] and builds a [`SaltDeclarationPayload`].
    pub fn finish(self) -> Result<SaltDeclarationPayload, ValidationError> {
        let node_id = self.node_id.ok_or(ValidationError::MissingField("node_id"))?;
        let salt = self.salt.ok_or(ValidationError::MissingField("salt"))?;
        let timestamp = self.timestamp.ok_or(ValidationError::MissingField("timestamp"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;

        Ok(SaltDeclarationPayload {
            node_id,
            salt,
            timestamp,
            signature,
        })
    }
}
