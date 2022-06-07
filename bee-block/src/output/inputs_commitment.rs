// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::hashes::{blake2b::Blake2b256, Digest};
use derive_more::{Deref, From};
use packable::PackableExt;

use crate::output::Output;

/// Represents a commitment to transaction inputs.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, From, Deref, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InputsCommitment([u8; Self::LENGTH]);

impl InputsCommitment {
    /// The length of an [`InputsCommitment`].
    pub const LENGTH: usize = 32;

    /// Creates a new [`InputsCommitment`] from a sequence of [`Output`]s.
    pub fn new<'a>(inputs: impl Iterator<Item = &'a Output>) -> Self {
        let mut hasher = Blake2b256::new();

        inputs.for_each(|output| hasher.update(Blake2b256::digest(&output.pack_to_vec())));

        Self(hasher.finalize().into())
    }
}

impl core::str::FromStr for InputsCommitment {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(InputsCommitment::from(
            prefix_hex::decode::<[u8; 32]>(s).map_err(crate::Error::HexError)?,
        ))
    }
}

impl core::fmt::Display for InputsCommitment {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", prefix_hex::encode(self.0))
    }
}

impl core::fmt::Debug for InputsCommitment {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "InputsCommitment({})", self)
    }
}
