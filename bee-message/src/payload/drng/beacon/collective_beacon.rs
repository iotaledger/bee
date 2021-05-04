// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH};
use crate::ValidationError;

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

use alloc::boxed::Box;
use core::convert::{Infallible, TryInto};

/// Message decsribing a dRNG `CollectiveBeacon`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectiveBeaconPayload {
    /// The version of the `CollectiveBeaconPayload`.
    version: u8,
    /// The identifier of the dRNG instance.
    instance_id: u32,
    /// The round of the current beacon.
    round: u64,
    /// The collective signature of the previous beacon.
    prev_signature: Box<[u8]>,
    /// The collective signature of the current beacon.
    signature: Box<[u8]>,
    /// The distributed public key.
    distributed_public_key: Box<[u8]>,
}

impl CollectiveBeaconPayload {
    /// The payload kind of a `CollectiveBeaconPayload`.
    pub const KIND: u32 = 6;

    /// Creates a new `CollectiveBeaconPayloadBuilder`.
    pub fn builder() -> CollectiveBeaconPayloadBuilder {
        CollectiveBeaconPayloadBuilder::default()
    }

    /// Returns the version of a `CollectiveBeaconPayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the instance ID of a `CollectiveBeaconPayload`.
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Returns the round of a `CollectiveBeaconPayload`.
    pub fn round(&self) -> u64 {
        self.round
    }

    /// Returns the signature of the previous beacon.
    pub fn prev_signature(&self) -> &[u8] {
        &self.prev_signature
    }

    /// Returns the signature of the current beacon.
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Returns the distributed public key of a `CollectiveBeaconPayload`.
    pub fn distributed_public_key(&self) -> &[u8] {
        &self.distributed_public_key
    }
}

impl Packable for CollectiveBeaconPayload {
    type PackError = Infallible;
    type UnpackError = Infallible;

    fn packed_len(&self) -> usize {
        self.version.packed_len()
            + self.instance_id.packed_len()
            + self.round.packed_len()
            + BEACON_SIGNATURE_LENGTH
            + BEACON_SIGNATURE_LENGTH
            + BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        self.version.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;
        self.round.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.prev_signature` is known to be 96 bytes.
        let prev_sig_bytes: [u8; BEACON_SIGNATURE_LENGTH] = self.prev_signature.to_vec().try_into().unwrap();
        prev_sig_bytes.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.signature` is known to be 96 bytes.
        let sig_bytes: [u8; BEACON_SIGNATURE_LENGTH] = self.signature.to_vec().try_into().unwrap();
        sig_bytes.pack(packer).map_err(PackError::infallible)?;

        // The size of `self.distributed_public_key` is known to be 48 bytes.
        let distributed_pk_bytes: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH] =
            self.distributed_public_key.to_vec().try_into().unwrap();
        distributed_pk_bytes.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let round = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let prev_signature = <[u8; BEACON_SIGNATURE_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();
        let signature = <[u8; BEACON_SIGNATURE_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();
        let distributed_public_key = <[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH]>::unpack(unpacker)
            .map_err(UnpackError::infallible)?
            .into();

        Ok(Self {
            version,
            instance_id,
            round,
            prev_signature,
            signature,
            distributed_public_key,
        })
    }
}

/// Builder that builds a `CollectiveBeaconPayload`.
#[derive(Default)]
pub struct CollectiveBeaconPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    round: Option<u64>,
    prev_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    distributed_public_key: Option<[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH]>,
}

impl CollectiveBeaconPayloadBuilder {
    /// Adds a version to a `CollectiveBeaconPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version.replace(version);
        self
    }

    /// Adds an instance ID to a `CollectiveBeaconPayloadBuilder`.
    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    /// Adds a round number to a `CollectiveBeaconPayloadBuilder`.
    pub fn with_round(mut self, round: u64) -> Self {
        self.round.replace(round);
        self
    }

    /// Returns the previous signature of a `CollectiveBeaconPayloadBuilder`.
    pub fn with_prev_signature(mut self, prev_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.prev_signature.replace(prev_signature);
        self
    }

    /// Returns the current signature of a `CollectiveBeaconPayloadBuilder`.
    pub fn with_signature(mut self, signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.signature.replace(signature);
        self
    }

    /// Returns the distributed public key of a `CollectiveBeaconPayloadBuilder`.
    pub fn with_distributed_public_key(
        mut self,
        distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
    ) -> Self {
        self.distributed_public_key.replace(distributed_public_key);
        self
    }

    /// Consumes the `CollectiveBeaconPayloadBuilder` and builds a new `CollectiveBeaconPayload`.
    pub fn finish(self) -> Result<CollectiveBeaconPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let prev_signature = self
            .prev_signature
            .ok_or(ValidationError::MissingField("prev_signature"))?;
        let signature = self.signature.ok_or(ValidationError::MissingField("signature"))?;
        let distributed_public_key = self
            .distributed_public_key
            .ok_or(ValidationError::MissingField("distributed_public_key"))?;

        Ok(CollectiveBeaconPayload {
            version,
            instance_id,
            round,
            prev_signature: prev_signature.into(),
            signature: signature.into(),
            distributed_public_key: distributed_public_key.into(),
        })
    }
}
