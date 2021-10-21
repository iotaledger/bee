// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_ternary::{T5B1Buf, TritBuf, Trits, T5B1};

use bytemuck::cast_slice;

/// The length of a tail transaction hash.
pub const TAIL_TRANSACTION_HASH_LEN: usize = 49;

/// Represents a tail transaction hash of a legacy bundle.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TailTransactionHash(TritBuf<T5B1Buf>);

impl TailTransactionHash {
    /// Creates a new `TailTransactionHash`.
    pub fn new(bytes: [u8; TAIL_TRANSACTION_HASH_LEN]) -> Result<Self, Error> {
        bytes.try_into()
    }
}

impl TryFrom<[u8; TAIL_TRANSACTION_HASH_LEN]> for TailTransactionHash {
    type Error = Error;

    fn try_from(bytes: [u8; TAIL_TRANSACTION_HASH_LEN]) -> Result<Self, Error> {
        Ok(TailTransactionHash(
            Trits::<T5B1>::try_from_raw(cast_slice(&bytes), 243)
                .map_err(|_| Error::InvalidTailTransactionHash)?
                .to_buf(),
        ))
    }
}

impl AsRef<[u8]> for TailTransactionHash {
    fn as_ref(&self) -> &[u8] {
        cast_slice(self.0.as_slice().as_i8_slice())
    }
}

impl core::fmt::Display for TailTransactionHash {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0.iter_trytes().map(char::from).collect::<String>())
    }
}

impl core::fmt::Debug for TailTransactionHash {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TailTransactionHash({})", self)
    }
}

impl Packable for TailTransactionHash {
    type Error = Error;

    fn packed_len(&self) -> usize {
        TAIL_TRANSACTION_HASH_LEN
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(self.as_ref())?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Self::new(<[u8; TAIL_TRANSACTION_HASH_LEN]>::unpack_inner::<R, CHECK>(reader)?)
    }
}
