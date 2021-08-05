// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::{
        drng::beacon::{BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH},
        MessagePayload,
    },
    MessageUnpackError, ValidationError,
};

use bee_packable::Packable;

/// Message decsribing a dRNG [`CollectiveBeaconPayload`].
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct CollectiveBeaconPayload {
    /// The identifier of the dRNG instance.
    instance_id: u32,
    /// The round of the current beacon.
    round: u64,
    /// The collective signature of the previous beacon.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    prev_signature: [u8; BEACON_SIGNATURE_LENGTH],
    /// The collective signature of the current beacon.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    signature: [u8; BEACON_SIGNATURE_LENGTH],
    /// The distributed public key.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
}

impl MessagePayload for CollectiveBeaconPayload {
    const KIND: u32 = 6;
    const VERSION: u8 = 0;
}

impl CollectiveBeaconPayload {
    /// Creates a new [`CollectiveBeaconPayloadBuilder`].
    pub fn builder() -> CollectiveBeaconPayloadBuilder {
        CollectiveBeaconPayloadBuilder::new()
    }

    /// Returns the instance ID of a [`CollectiveBeaconPayload`].
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Returns the round of a [`CollectiveBeaconPayload`].
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

    /// Returns the distributed public key of a [`CollectiveBeaconPayload`].
    pub fn distributed_public_key(&self) -> &[u8] {
        &self.distributed_public_key
    }
}

/// Builder that builds a [`CollectiveBeaconPayload`].
#[derive(Default)]
pub struct CollectiveBeaconPayloadBuilder {
    instance_id: Option<u32>,
    round: Option<u64>,
    prev_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
    distributed_public_key: Option<[u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH]>,
}

impl CollectiveBeaconPayloadBuilder {
    /// Creates a new [`CollectiveBeaconPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instance ID to a [`CollectiveBeaconPayloadBuilder`].
    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    /// Adds a round number to a [`CollectiveBeaconPayloadBuilder`].
    pub fn with_round(mut self, round: u64) -> Self {
        self.round.replace(round);
        self
    }

    /// Returns the previous signature of a [`CollectiveBeaconPayloadBuilder`].
    pub fn with_prev_signature(mut self, prev_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.prev_signature.replace(prev_signature);
        self
    }

    /// Returns the current signature of a [`CollectiveBeaconPayloadBuilder`].
    pub fn with_signature(mut self, signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.signature.replace(signature);
        self
    }

    /// Returns the distributed public key of a [`CollectiveBeaconPayloadBuilder`].
    pub fn with_distributed_public_key(
        mut self,
        distributed_public_key: [u8; BEACON_DISTRIBUTED_PUBLIC_KEY_LENGTH],
    ) -> Self {
        self.distributed_public_key.replace(distributed_public_key);
        self
    }

    /// Consumes the [`CollectiveBeaconPayloadBuilder`] and builds a new [`CollectiveBeaconPayload`].
    pub fn finish(self) -> Result<CollectiveBeaconPayload, ValidationError> {
        let instance_id = self
            .instance_id
            .ok_or(ValidationError::MissingBuilderField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingBuilderField("round"))?;
        let prev_signature = self
            .prev_signature
            .ok_or(ValidationError::MissingBuilderField("prev_signature"))?;
        let signature = self
            .signature
            .ok_or(ValidationError::MissingBuilderField("signature"))?;
        let distributed_public_key = self
            .distributed_public_key
            .ok_or(ValidationError::MissingBuilderField("distributed_public_key"))?;

        Ok(CollectiveBeaconPayload {
            instance_id,
            round,
            prev_signature,
            signature,
            distributed_public_key,
        })
    }
}
