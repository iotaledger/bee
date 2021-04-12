// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

/// The length of the BLAKE2b-256 hash output.
///
/// See <https://www.blake2.net/> for more information.
pub const MESSAGE_ID_LENGTH: usize = 32;

/// The BLAKE2b-256 hash of the byte contents of the message.
///
/// Use `crypto::hashes::Blake2b256::digest` to produce the hashed value.
///
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct MessageId([u8; MESSAGE_ID_LENGTH]);

impl MessageId {
    /// Construct a new `MessageId` from the BLAKE2b-256 hash of a message.
    pub fn new(bytes: [u8; MESSAGE_ID_LENGTH]) -> Self {
        bytes.into()
    }

    /// Create a null `MessageId` (all zeros).
    pub fn null() -> Self {
        Self([0u8; MESSAGE_ID_LENGTH])
    }
}

#[cfg(feature = "serde")]
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

impl Packable for MessageId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; MESSAGE_ID_LENGTH];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
