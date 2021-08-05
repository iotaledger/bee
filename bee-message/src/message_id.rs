// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, util::hex_decode};

use bee_packable::packable::Packable;

/// A [`Message`](crate::Message) identifier, the BLAKE2b-256 hash of the message bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageId([u8; Self::LENGTH]);

impl MessageId {
    /// The length, in bytes, of a [`MessageId`].
    pub const LENGTH: usize = 32;

    /// Creates a new [`MessageId`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    /// Create a null [`MessageId`].
    pub fn null() -> Self {
        Self([0u8; Self::LENGTH])
    }
}

impl From<[u8; Self::LENGTH]> for MessageId {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl AsRef<[u8]> for MessageId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::ops::Deref for MessageId {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::str::FromStr for MessageId {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(MessageId::from(hex_decode(hex)?))
    }
}

impl core::fmt::Display for MessageId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for MessageId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MessageId({})", self)
    }
}
