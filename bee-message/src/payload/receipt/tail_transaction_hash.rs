// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::string::String;
use core::fmt;

use bee_ternary::{T5B1Buf, TritBuf, Trits, T5B1};
use bytemuck::cast_slice;
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use crate::Error;

/// Represents a tail transaction hash of a legacy bundle.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TailTransactionHash(TritBuf<T5B1Buf>);

impl TailTransactionHash {
    /// The length of a [`TailTransactionHash`].
    pub const LENGTH: usize = 49;

    /// Creates a new [`TailTransactionHash`].
    pub fn new(bytes: [u8; TailTransactionHash::LENGTH]) -> Result<Self, Error> {
        bytes.try_into()
    }
}

impl TryFrom<[u8; TailTransactionHash::LENGTH]> for TailTransactionHash {
    type Error = Error;

    fn try_from(bytes: [u8; TailTransactionHash::LENGTH]) -> Result<Self, Error> {
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

impl fmt::Display for TailTransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.iter_trytes().map(char::from).collect::<String>())
    }
}

impl fmt::Debug for TailTransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TailTransactionHash({})", self)
    }
}

impl Packable for TailTransactionHash {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        packer.pack_bytes(self.as_ref())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        Self::new(<[u8; TailTransactionHash::LENGTH]>::unpack::<_, VERIFY>(unpacker).infallible()?)
            .map_err(UnpackError::Packable)
    }
}
