// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{MessagePackError, MessageUnpackError, ValidationError},
    payload::PAYLOAD_LENGTH_MAX,
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

/// All `Vec` sizes are unconstrained, so use payload max as upper limit.
const PREFIXED_LENGTH_MAX: usize = PAYLOAD_LENGTH_MAX;

/// Error packing a DKG payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum DkgPackError {
    InvalidPrefix,
}

impl From<PackPrefixError<Infallible, u32>> for DkgPackError {
    fn from(error: PackPrefixError<Infallible, u32>) -> Self {
        match error {
            PackPrefixError::Packable(e) => match e {},
            PackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for DkgPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for encrypted deal data"),
        }
    }
}

/// Error unpacking a DKG payload.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum DkgUnpackError {
    InvalidPrefix,
    InvalidPrefixLength(usize),
}

impl_from_infallible!(DkgUnpackError);

impl From<UnpackPrefixError<Infallible, u32>> for DkgUnpackError {
    fn from(error: UnpackPrefixError<Infallible, u32>) -> Self {
        match error {
            UnpackPrefixError::InvalidPrefixLength(len) => Self::InvalidPrefixLength(len),
            UnpackPrefixError::Packable(e) => match e {},
            UnpackPrefixError::Prefix(_) => Self::InvalidPrefix,
        }
    }
}

impl fmt::Display for DkgUnpackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix => write!(f, "invalid prefix for salt bytes"),
            Self::InvalidPrefixLength(len) => write!(f, "unpacked prefix larger than maximum specified: {}", len),
        }
    }
}

/// Encrypted share structure for a `DkgPayload`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct EncryptedDeal {
    /// An ephemeral Diffie-Hellman key.
    dh_key: Vec<u8>,
    /// The nonce used.
    nonce: Vec<u8>,
    /// The ciphertext of the share.
    encrypted_share: Vec<u8>,
    /// The threshold of the secret sharing protocol.
    threshold: u32,
    /// The commitments of the polynomial used to derive the share.
    commitments: Vec<u8>,
}

impl EncryptedDeal {
    /// Creates a new `EncryptedDealBuilder`.
    pub fn builder() -> EncryptedDealBuilder {
        EncryptedDealBuilder::new()
    }

    /// Returns the Diffie-Hellman key of the `EncryptedDeal`.
    pub fn dh_key(&self) -> &[u8] {
        self.dh_key.as_slice()
    }

    /// Returns the nonce of the `EncryptedDeal`.
    pub fn nonce(&self) -> &[u8] {
        self.nonce.as_slice()
    }

    /// Returns the encrypted share of the `EncryptedDeal`.
    pub fn encrypted_share(&self) -> &[u8] {
        self.encrypted_share.as_slice()
    }

    /// Returns the threshold of the `EncryptedDeal`.
    pub fn threshold(&self) -> u32 {
        self.threshold
    }

    /// Adds commitments to an `EncryptedDealBuilder`.
    pub fn commitments(&self) -> &[u8] {
        self.commitments.as_slice()
    }
}

impl Packable for EncryptedDeal {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        // Unwraps are safe, since the `EncryptedDeal` length has been validated.
        let prefixed_dh_key: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> = self.dh_key.clone().try_into().unwrap();
        let prefixed_nonce: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> = self.nonce.clone().try_into().unwrap();
        let prefixed_encrypted_share: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> =
            self.encrypted_share.clone().try_into().unwrap();
        let prefixed_commitments: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> =
            self.commitments.clone().try_into().unwrap();

        prefixed_dh_key.packed_len()
            + prefixed_nonce.packed_len()
            + prefixed_encrypted_share.packed_len()
            + self.threshold.packed_len()
            + prefixed_commitments.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        // The following unwraps are safe, since the `EncryptedDeal` length has been validated.
        let prefixed_dh_key: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> = self.dh_key.clone().try_into().unwrap();
        prefixed_dh_key
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_nonce: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> = self.nonce.clone().try_into().unwrap();
        prefixed_nonce
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        let prefixed_encrypted_share: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> =
            self.encrypted_share.clone().try_into().unwrap();
        prefixed_encrypted_share
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        self.threshold.pack(packer).map_err(PackError::infallible)?;

        let prefixed_commitments: VecPrefix<u8, u32, PREFIXED_LENGTH_MAX> =
            self.commitments.clone().try_into().unwrap();
        prefixed_commitments
            .pack(packer)
            .map_err(PackError::coerce::<DkgPackError>)
            .map_err(PackError::coerce)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let dh_key = VecPrefix::<u8, u32, PREFIXED_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let nonce = VecPrefix::<u8, u32, PREFIXED_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let encrypted_share = VecPrefix::<u8, u32, PREFIXED_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let threshold = u32::unpack(unpacker).map_err(UnpackError::infallible)?;

        let commitments = VecPrefix::<u8, u32, PREFIXED_LENGTH_MAX>::unpack(unpacker)
            .map_err(UnpackError::coerce::<DkgUnpackError>)
            .map_err(UnpackError::coerce)?
            .into();

        let deal = EncryptedDeal {
            dh_key,
            nonce,
            encrypted_share,
            threshold,
            commitments,
        };

        validate_encrypted_deal_length(deal.packed_len()).map_err(|e| UnpackError::Packable(e.into()))?;

        Ok(deal)
    }
}

/// A builder that builds an `EncryptedDeal`.
#[derive(Default)]
pub struct EncryptedDealBuilder {
    dh_key: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    encrypted_share: Option<Vec<u8>>,
    threshold: Option<u32>,
    commitments: Option<Vec<u8>>,
}

impl EncryptedDealBuilder {
    /// Creates a new `EncryptedDeal`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a Diffie-Hellman key to an `EncryptedDeal`.
    pub fn with_dh_key(mut self, dh_key: Vec<u8>) -> Self {
        self.dh_key.replace(dh_key);
        self
    }

    /// Adds a nonce to an `EncryptedDeal`.
    pub fn with_nonce(mut self, nonce: Vec<u8>) -> Self {
        self.nonce.replace(nonce);
        self
    }

    /// Adds an encrypted share to an `EncryptedDeal`.
    pub fn with_encrypted_share(mut self, encrypted_share: Vec<u8>) -> Self {
        self.encrypted_share.replace(encrypted_share);
        self
    }

    /// Adds a threshold to an `EncryptedDeal`.
    pub fn with_threshold(mut self, threshold: u32) -> Self {
        self.threshold.replace(threshold);
        self
    }

    /// Adds commitments to an `EncryptedDeal`.
    pub fn with_commitments(mut self, commitments: Vec<u8>) -> Self {
        self.commitments.replace(commitments);
        self
    }

    /// Consumes the `EncryptedDealBuilder` and builds a new `EncryptedDeal`.
    pub fn finish(self) -> Result<EncryptedDeal, ValidationError> {
        let dh_key = self.dh_key.ok_or(ValidationError::MissingField("dh_key"))?;
        let nonce = self.nonce.ok_or(ValidationError::MissingField("nonce"))?;
        let encrypted_share = self
            .encrypted_share
            .ok_or(ValidationError::MissingField("encrypted_share"))?;
        let threshold = self.threshold.ok_or(ValidationError::MissingField("threshold"))?;
        let commitments = self.commitments.ok_or(ValidationError::MissingField("commitments"))?;

        let deal = EncryptedDeal {
            dh_key,
            nonce,
            encrypted_share,
            threshold,
            commitments,
        };

        validate_encrypted_deal_length(deal.packed_len())?;

        Ok(deal)
    }
}

fn validate_encrypted_deal_length(len: usize) -> Result<(), ValidationError> {
    if len > PREFIXED_LENGTH_MAX {
        Err(ValidationError::InvalidEncryptedDealLength(len))
    } else {
        Ok(())
    }
}

/// The `Deal` messages exchanged to produce a public/private collective key during the DKG phase of dRNG.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct DkgPayload {
    /// The version of the `DkgPayload`.
    version: u8,
    /// The identifier of the dRNG instance.
    instance_id: u32,
    /// The index of the dealer.
    from_index: u32,
    /// The index of the verifier.
    to_index: u32,
    /// The encrypted share struct.
    deal: EncryptedDeal,
}

impl DkgPayload {
    /// The payload kind of a `DkgPayload`.
    pub const KIND: u32 = 4;

    /// Creates a new `DkgPayloadBuilder`.
    pub fn builder() -> DkgPayloadBuilder {
        DkgPayloadBuilder::new()
    }

    /// Returns the version of a `DkgPayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the instance ID of a `DkgPayload`.
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Returns the "from index" of a `DkgPayload`.
    pub fn from_index(&self) -> u32 {
        self.from_index
    }

    /// Returns the "to index" of a `DkgPayload`.
    pub fn to_index(&self) -> u32 {
        self.to_index
    }

    /// Returns the encrypted deal of a `DkgPayload`.
    pub fn deal(&self) -> &EncryptedDeal {
        &self.deal
    }
}

impl Packable for DkgPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.instance_id.packed_len()
            + self.from_index.packed_len()
            + self.to_index.packed_len()
            + self.deal.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;
        self.from_index.pack(packer).map_err(PackError::infallible)?;
        self.to_index.pack(packer).map_err(PackError::infallible)?;
        self.deal.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let from_index = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let to_index = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let deal = EncryptedDeal::unpack(unpacker)?;

        Ok(Self {
            version,
            instance_id,
            from_index,
            to_index,
            deal,
        })
    }
}

/// A builder that builds a `DkgPayload`.
#[derive(Default)]
pub struct DkgPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    from_index: Option<u32>,
    to_index: Option<u32>,
    deal: Option<EncryptedDeal>,
}

impl DkgPayloadBuilder {
    /// Creates a new `DkgPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a version to a `DkgPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version.replace(version);
        self
    }

    /// Adds an instance ID to a `DkgPayloadBuilder`.
    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    /// Adds the dealer index to a `DkgPayloadBuilder`.
    pub fn with_from_index(mut self, from_index: u32) -> Self {
        self.from_index.replace(from_index);
        self
    }

    /// Adds the verifier index to a `DkgPayloadBuilder`.
    pub fn with_to_index(mut self, to_index: u32) -> Self {
        self.to_index.replace(to_index);
        self
    }

    /// Adds an encrypted deal to a `DkgPayloadBuilder`.
    pub fn with_deal(mut self, deal: EncryptedDeal) -> Self {
        self.deal.replace(deal);
        self
    }

    /// Consumes the `DkgPayloadBuilder` and builds a new `DkgPayload`.
    pub fn finish(self) -> Result<DkgPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let from_index = self.from_index.ok_or(ValidationError::MissingField("from_index"))?;
        let to_index = self.to_index.ok_or(ValidationError::MissingField("to_index"))?;
        let deal = self.deal.ok_or(ValidationError::MissingField("deal"))?;

        Ok(DkgPayload {
            version,
            instance_id,
            from_index,
            to_index,
            deal,
        })
    }
}
