// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::error::ValidationError;

use bee_common::packable::Packable;

use alloc::borrow::ToOwned;
use core::{convert::TryInto, str::FromStr};

/// The length of a transaction identifier.
pub const TRANSACTION_ID_LENGTH: usize = 32;

/// A transaction identifier, the BLAKE2b-256 hash of the transaction bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionId([u8; TRANSACTION_ID_LENGTH]);

impl TransactionId {
    /// Creates a new `TransactionId`.
    pub fn new(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        bytes.into()
    }
}

impl From<[u8; TRANSACTION_ID_LENGTH]> for TransactionId {
    fn from(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for TransactionId {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; TRANSACTION_ID_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(TRANSACTION_ID_LENGTH * 2, s.len()))?;

        Ok(TransactionId::from(bytes))
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
