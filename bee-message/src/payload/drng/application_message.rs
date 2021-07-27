// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::MessagePayload, MessagePackError, MessageUnpackError, ValidationError};

use bee_packable::{PackError, Packable, Packer, UnpackError, Unpacker};

/// Message used by a node to declare its willingness to participate in the Committee Selection process.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationMessagePayload {
    /// The identifier of the dRNG instance.
    instance_id: u32,
}

impl MessagePayload for ApplicationMessagePayload {
    const KIND: u32 = 3;
    const VERSION: u8 = 0;
}

impl ApplicationMessagePayload {
    /// Creates a new [`ApplicationMessagePayload`].
    pub fn new(instance_id: u32) -> Self {
        Self { instance_id }
    }

    /// Returns the instance ID of an [`ApplicationMessagePayload`].
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }
}

impl Packable for ApplicationMessagePayload {
    type PackError = MessagePackError;
    type UnpackError = MessageUnpackError;

    fn packed_len(&self) -> usize {
        Self::VERSION.packed_len() + self.instance_id.packed_len()
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
        Self::VERSION.pack(packer).map_err(PackError::infallible)?;
        self.instance_id.pack(packer).map_err(PackError::infallible)?;

        Ok(())
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack(unpacker).map_err(UnpackError::infallible)?;
        validate_payload_version(version).map_err(|e| UnpackError::Packable(e.into()))?;

        let instance_id = u32::unpack(unpacker).map_err(UnpackError::infallible)?;

        Ok(Self { instance_id })
    }
}

fn validate_payload_version(version: u8) -> Result<(), ValidationError> {
    if version != ApplicationMessagePayload::VERSION {
        Err(ValidationError::InvalidPayloadVersion(
            version,
            ApplicationMessagePayload::KIND,
        ))
    } else {
        Ok(())
    }
}
