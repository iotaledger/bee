// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_packable::Packable;

use core::str::FromStr;

/// The length of a message identifier.
pub const MESSAGE_ID_LENGTH: usize = 32;

/// A message identifier, the BLAKE2b-256 hash of the message bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Packable)]
pub struct MessageId([u8; MESSAGE_ID_LENGTH]);

impl MessageId {
    /// Creates a new `MessageId`.
    pub fn new(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        bytes.into()
    }

    /// Create a null `MessageId`.
    pub fn null() -> Self {
        Self([0u8; MESSAGE_ID_LENGTH])
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(MessageId);

impl From<[u8; MESSAGE_ID_LENGTH]> for MessageId {
    fn from(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for MessageId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; MESSAGE_ID_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(MESSAGE_ID_LENGTH * 2, s.len()))?;

        Ok(MessageId::from(bytes))
    }
}

impl AsRef<[u8]> for MessageId {
    fn as_ref(&self) -> &[u8] {
        &self.0
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
