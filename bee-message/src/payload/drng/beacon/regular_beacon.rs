// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{BEACON_PARTIAL_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH};
use crate::ValidationError;

use bee_packable::Packable;

/// Message representing a dRNG `Beacon`.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconPayload {
    /// The version of the `BeaconPayload`.
    version: u8,
    /// The identifier of the dRNG instance.
    instance_id: u32,
    /// The round of the current beacon.
    round: u64,
    /// The public key of the issuer.
    #[cfg_attr(feature = "enable-serde", serde(with = "serde_big_array::BigArray"))]
    partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH],
    /// The collective signature of the current beacon.
    #[cfg_attr(feature = "enable-serde", serde(with = "serde_big_array::BigArray"))]
    partial_signature: [u8; BEACON_SIGNATURE_LENGTH],
}

impl BeaconPayload {
    /// The payload kind of a `BeaconPayload`.
    pub const KIND: u32 = 5;

    /// Creates a new `BeaconPayloadBuilder`.
    pub fn builder() -> BeaconPayloadBuilder {
        BeaconPayloadBuilder::new()
    }

    /// Returns the version of a `BeaconPayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the instance ID of a `BeaconPayload`.
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Returns the round number of a `BeaconPayload`.
    pub fn round(&self) -> u64 {
        self.round
    }

    /// Returns the partial public key of a `BeaconPayload`.
    pub fn partial_public_key(&self) -> &[u8] {
        &self.partial_public_key
    }

    /// Returns the partial signature of a `BeaconPayload`.
    pub fn partial_signature(&self) -> &[u8] {
        &self.partial_signature
    }
}

/// Builder than builds a `BeaconPayload`.
#[derive(Default)]
pub struct BeaconPayloadBuilder {
    version: Option<u8>,
    instance_id: Option<u32>,
    round: Option<u64>,
    partial_public_key: Option<[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]>,
    partial_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
}

impl BeaconPayloadBuilder {
    /// Creates a new `BeaconPayloadBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a version number to a `BeaconPayloadBuilder`.
    pub fn with_version(mut self, version: u8) -> Self {
        self.version.replace(version);
        self
    }

    /// Adds an instance ID to a `BeaconPayloadBuilder`.
    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    /// Adds a round number to a `BeaconPayloadBuilder`.
    pub fn with_round(mut self, round: u64) -> Self {
        self.round.replace(round);
        self
    }

    /// Adds a partial public key to a `BeaconPayloadBuilder`.
    pub fn with_partial_public_key(mut self, partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]) -> Self {
        self.partial_public_key.replace(partial_public_key);
        self
    }

    /// Adds a partial signature to a `BeaconPayloadBuilder`.
    pub fn with_partial_signature(mut self, partial_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.partial_signature.replace(partial_signature);
        self
    }

    /// Consumes the `BeaconPayloadBuilder` and builds a new `BeaconPayload`.
    pub fn finish(self) -> Result<BeaconPayload, ValidationError> {
        let version = self.version.ok_or(ValidationError::MissingField("version"))?;
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let partial_public_key = self
            .partial_public_key
            .ok_or(ValidationError::MissingField("partial_public_key"))?;
        let partial_signature = self
            .partial_signature
            .ok_or(ValidationError::MissingField("partial_signature"))?;

        Ok(BeaconPayload {
            version,
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
    }
}
