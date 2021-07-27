// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::{
        drng::beacon::{BEACON_PARTIAL_PUBLIC_KEY_LENGTH, BEACON_SIGNATURE_LENGTH},
        MessagePayload,
    },
    MessagePackError, MessageUnpackError, ValidationError,
};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

/// Message representing a dRNG [`BeaconPayload`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct BeaconPayload {
    /// The identifier of the dRNG instance.
    instance_id: u32,
    /// The round of the current beacon.
    round: u64,
    /// The public key of the issuer.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH],
    /// The collective signature of the current beacon.
    #[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))]
    partial_signature: [u8; BEACON_SIGNATURE_LENGTH],
}

impl MessagePayload for BeaconPayload {
    const KIND: u32 = 5;
    const VERSION: u8 = 0;
}

impl BeaconPayload {
    /// Creates a new [`BeaconPayloadBuilder`].
    pub fn builder() -> BeaconPayloadBuilder {
        BeaconPayloadBuilder::new()
    }

    /// Returns the instance ID of a [`BeaconPayload`].
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    /// Returns the round number of a [`BeaconPayload`].
    pub fn round(&self) -> u64 {
        self.round
    }

    /// Returns the partial public key of a [`BeaconPayload`].
    pub fn partial_public_key(&self) -> &[u8] {
        &self.partial_public_key
    }

    /// Returns the partial signature of a [`BeaconPayload`].
    pub fn partial_signature(&self) -> &[u8] {
        &self.partial_signature
    }
}

impl Packable for BeaconPayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        Self::VERSION.packed_len()
            + self.instance_id.packed_len()
            + self.round.packed_len()
            + self.partial_public_key.packed_len()
            + self.partial_signature.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        Self::VERSION.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;
        self.round.pack(packer).map_err(PackError::infallible)?;
        self.partial_public_key.pack(packer).map_err(PackError::infallible)?;
        self.partial_signature.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_payload_version(version).map_err(|e| UnpackError::Packable(e.into()))?;

        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;
        let round = u64::unpack(unpacker).map_err(UnpackError::infallible)?;
        let partial_public_key =
            <[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;
        let partial_signature = <[u8; BEACON_SIGNATURE_LENGTH]>::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self {
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
    }
}

fn validate_payload_version(version: u8) -> Result<(), ValidationError> {
    if version != BeaconPayload::VERSION {
        Err(ValidationError::InvalidPayloadVersion(version, BeaconPayload::KIND))
    } else {
        Ok(())
    }
}

/// Builder than builds a [`BeaconPayload`].
#[derive(Default)]
pub struct BeaconPayloadBuilder {
    instance_id: Option<u32>,
    round: Option<u64>,
    partial_public_key: Option<[u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]>,
    partial_signature: Option<[u8; BEACON_SIGNATURE_LENGTH]>,
}

impl BeaconPayloadBuilder {
    /// Creates a new [`BeaconPayloadBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instance ID to a [`BeaconPayloadBuilder`].
    pub fn with_instance_id(mut self, instance_id: u32) -> Self {
        self.instance_id.replace(instance_id);
        self
    }

    /// Adds a round number to a [`BeaconPayloadBuilder`].
    pub fn with_round(mut self, round: u64) -> Self {
        self.round.replace(round);
        self
    }

    /// Adds a partial public key to a [`BeaconPayloadBuilder`].
    pub fn with_partial_public_key(mut self, partial_public_key: [u8; BEACON_PARTIAL_PUBLIC_KEY_LENGTH]) -> Self {
        self.partial_public_key.replace(partial_public_key);
        self
    }

    /// Adds a partial signature to a [`BeaconPayloadBuilder`].
    pub fn with_partial_signature(mut self, partial_signature: [u8; BEACON_SIGNATURE_LENGTH]) -> Self {
        self.partial_signature.replace(partial_signature);
        self
    }

    /// Consumes the [`BeaconPayloadBuilder`] and builds a new [`BeaconPayload`].
    pub fn finish(self) -> Result<BeaconPayload, ValidationError> {
        let instance_id = self.instance_id.ok_or(ValidationError::MissingField("instance_id"))?;
        let round = self.round.ok_or(ValidationError::MissingField("round"))?;
        let partial_public_key = self
            .partial_public_key
            .ok_or(ValidationError::MissingField("partial_public_key"))?;
        let partial_signature = self
            .partial_signature
            .ok_or(ValidationError::MissingField("partial_signature"))?;

        Ok(BeaconPayload {
            instance_id,
            round,
            partial_public_key,
            partial_signature,
        })
    }
}
