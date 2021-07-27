// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{error::ValidationError, util::hex_decode};

use bee_packable::packable::Packable;

use core::str::FromStr;

/// A transaction identifier, the BLAKE2b-256 hash of the transaction bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionId([u8; Self::LENGTH]);

impl TransactionId {
    /// The length (in bytes) of a [`TransactionId`].
    pub const LENGTH: usize = 32;

    /// Creates a new [`TransactionId`].
    pub fn new(bytes: [u8; Self::LENGTH]) -> Self {
        bytes.into()
    }
}

impl From<[u8; Self::LENGTH]> for TransactionId {
    fn from(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for TransactionId {
    type Err = ValidationError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        Ok(TransactionId::from(hex_decode::<{ Self::LENGTH }>(hex)?))
    }
}

impl AsRef<[u8]> for TransactionId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for TransactionId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TransactionId({})", self)
    }
}
