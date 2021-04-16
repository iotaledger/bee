// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

/// The length of a transaction identifier.
pub const TRANSACTION_ID_LENGTH: usize = 32;

/// A transaction identifier, the BLAKE2b-256 hash of the transaction bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct TransactionId([u8; TRANSACTION_ID_LENGTH]);

impl TransactionId {
    /// Creates a new `TransactionId`.
    pub fn new(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        bytes.into()
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(TransactionId);

impl From<[u8; TRANSACTION_ID_LENGTH]> for TransactionId {
    fn from(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for TransactionId {
    type Err = Error;

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

impl Packable for TransactionId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        TRANSACTION_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(<[u8; TRANSACTION_ID_LENGTH]>::unpack_inner::<R, CHECK>(reader)?))
    }
}
