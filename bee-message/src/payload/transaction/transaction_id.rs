// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use core::{convert::TryInto, str::FromStr};

pub const TRANSACTION_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize, Ord, PartialOrd)]
pub struct TransactionId([u8; TRANSACTION_ID_LENGTH]);

impl From<[u8; TRANSACTION_ID_LENGTH]> for TransactionId {
    fn from(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for TransactionId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; TRANSACTION_ID_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHex)?
            .as_slice()
            .try_into()
            .map_err(|_| Self::Err::InvalidHex)?;

        Ok(TransactionId::from(bytes))
    }
}

impl AsRef<[u8]> for TransactionId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TransactionId {
    pub fn new(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        bytes.into()
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
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; TRANSACTION_ID_LENGTH];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
