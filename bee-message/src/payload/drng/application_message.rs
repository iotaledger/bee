// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::payload::MessagePayload;

use bee_packable::Packable;

/// Message used by a node to declare its willingness to participate in the Committee Selection process.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
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
