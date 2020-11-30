// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_ternary::{T5B1Buf, TritBuf};

use bytemuck::cast_slice;
use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, string::String};
use core::convert::{TryFrom, TryInto};

// TODO length is 243, change to array when std::array::LengthAtMost32 disappears.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
pub struct WotsAddress(Box<[u8]>);

impl TryFrom<&TritBuf<T5B1Buf>> for WotsAddress {
    type Error = Error;

    fn try_from(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        // TODO const
        if trits.len() != 243 {
            return Err(Error::InvalidAddress);
        }

        Ok(Self(cast_slice(trits.as_i8_slice()).to_vec().into_boxed_slice()))
    }
}

impl AsRef<[u8]> for WotsAddress {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

// TODO builder ?
impl WotsAddress {
    pub fn new(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        trits.try_into()
    }

    pub fn to_bech32(&self) -> String {
        // TODO
        String::from("")
    }
}

impl core::fmt::Display for WotsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.to_bech32())
    }
}

impl core::fmt::Debug for WotsAddress {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "WotsAddress({})", self)
    }
}

impl Packable for WotsAddress {
    type Error = Error;

    fn packed_len(&self) -> usize {
        243
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; 243];
        reader.read_exact(&mut bytes)?;

        Ok(Self(Box::new(bytes)))
    }
}
