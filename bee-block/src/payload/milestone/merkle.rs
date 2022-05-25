// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

/// A Merkle root of a list of hashes.
#[derive(Clone, Copy, Eq, PartialEq, packable::Packable, derive_more::From, derive_more::AsRef)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MerkleRoot([u8; Self::LENGTH]);

impl MerkleRoot {
    /// Length of a merkle root.
    pub const LENGTH: usize = 32;

    /// Creates a null [`MerkleRoot`].
    pub fn null() -> Self {
        Self::from([0u8; Self::LENGTH])
    }
}

impl core::ops::Deref for MerkleRoot {
    type Target = [u8; Self::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for MerkleRoot {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.0))
    }
}

impl core::fmt::Debug for MerkleRoot {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MerkleRoot({})", self)
    }
}

impl core::str::FromStr for MerkleRoot {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(
            prefix_hex::decode::<[u8; Self::LENGTH]>(s).map_err(Error::HexError)?,
        ))
    }
}
