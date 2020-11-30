// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

pub const MESSAGE_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct MessageId([u8; MESSAGE_ID_LENGTH]);

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
            .map_err(|_| Self::Err::InvalidHex)?
            .as_slice()
            .try_into()
            .map_err(|_| Self::Err::InvalidHex)?;

        Ok(MessageId::from(bytes))
    }
}

impl AsRef<[u8]> for MessageId {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl MessageId {
    pub fn new(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        bytes.into()
    }

    pub fn null() -> Self {
        Self([0u8; MESSAGE_ID_LENGTH])
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

impl Packable for MessageId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; MESSAGE_ID_LENGTH];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
