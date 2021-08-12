// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, util::hex_decode};

use bee_packable::Packable;

/// An [`IndexationPayload`](crate::payload::indexation::IndexationPayload) index padded with `0` up to the maximum
/// length.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct PaddedIndex(#[cfg_attr(feature = "serde1", serde(with = "serde_big_array::BigArray"))] [u8; Self::LENGTH]);

impl PaddedIndex {
    /// The length, in bytes, of a [`PaddedIndex`].
    pub const LENGTH: usize = 64;

    /// Creates a new [`PaddedIndex`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }
}

impl From<[u8; Self::LENGTH]> for PaddedIndex {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self::new(bytes)
    }
}

impl AsRef<[u8]> for PaddedIndex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl core::ops::Deref for PaddedIndex {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::str::FromStr for PaddedIndex {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(PaddedIndex::from(hex_decode(hex)?))
    }
}

impl core::fmt::Display for PaddedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for PaddedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "PaddedIndex({})", self)
    }
}
