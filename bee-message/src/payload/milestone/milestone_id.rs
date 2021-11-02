// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use core::{convert::TryInto, str::FromStr};

/// The length of a milestone identifier.
pub const MILESTONE_ID_LENGTH: usize = 32;

/// A milestone identifier, the BLAKE2b-256 hash of the milestone bytes.
/// See <https://www.blake2.net/> for more information.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct MilestoneId([u8; MILESTONE_ID_LENGTH]);

impl MilestoneId {
    /// Creates a new `MilestoneId`.
    pub fn new(bytes: [u8; MILESTONE_ID_LENGTH]) -> Self {
        bytes.into()
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(MilestoneId);

impl From<[u8; MILESTONE_ID_LENGTH]> for MilestoneId {
    fn from(bytes: [u8; MILESTONE_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl FromStr for MilestoneId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; MILESTONE_ID_LENGTH] = hex::decode(s)
            .map_err(|_| Self::Err::InvalidHexadecimalChar(s.to_owned()))?
            .try_into()
            .map_err(|_| Self::Err::InvalidHexadecimalLength(MILESTONE_ID_LENGTH * 2, s.len()))?;

        Ok(MilestoneId::from(bytes))
    }
}

impl AsRef<[u8]> for MilestoneId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl core::fmt::Display for MilestoneId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for MilestoneId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MilestoneId({})", self)
    }
}

impl Packable for MilestoneId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        MILESTONE_ID_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(<[u8; MILESTONE_ID_LENGTH]>::unpack_inner::<R, CHECK>(
            reader,
        )?))
    }
}
