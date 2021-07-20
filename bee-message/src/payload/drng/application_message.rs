// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_packable::packable::Packable;

/// Message used by a node to declare its willingness to participate in the Committee Selection process.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplicationMessagePayload {
    /// The version of the `ApplicationMessagePayload`.
    version: u8,
    /// The identifier of the dRNG instance.
    instance_id: u32,
}

impl ApplicationMessagePayload {
    /// The payload kind of an `ApplicationMessagePayload`.
    pub const KIND: u32 = 3;

    /// Creates a new `ApplicationMessagePayload`.
    pub fn new(version: u8, instance_id: u32) -> Self {
        Self { version, instance_id }
    }

    /// Returns the version of an `ApplicationMessagePayload`.
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the instance ID of an `ApplicationMesssagePayload`.
    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }
}
