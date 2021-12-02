// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::str::FromStr;

/// A message identifier, the BLAKE2b-256 hash of the message bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd, derive_more::From)]
pub struct MessageId([u8; MessageId::LENGTH]);

impl MessageId {
    /// The length of a [`MessageId`].
    pub const LENGTH: usize = 32;

    /// Creates a new [`MessageId`].
    pub fn new(bytes: [u8; MessageId::LENGTH]) -> Self {
        bytes.into()
    }

    /// Create a null [`MessageId`].
    pub fn null() -> Self {
        Self([0u8; MessageId::LENGTH])
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(MessageId);

impl FromStr for MessageId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; MessageId::LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(MessageId::LENGTH * 2, s.len()))?;

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
        MessageId::LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(<[u8; MessageId::LENGTH]>::unpack_inner::<R, CHECK>(reader)?))
    }
}
