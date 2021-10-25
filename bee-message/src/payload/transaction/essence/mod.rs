// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

pub use regular::{RegularEssence, RegularEssenceBuilder};

use crate::Error;

use bee_packable::{Packable, PackableExt};

use crypto::hashes::{blake2b::Blake2b256, Digest};

/// A generic essence that can represent different types defining transaction essences.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(tag_type = u8, with_error = Error::InvalidEssenceKind)]
#[packable(unpack_error = Error)]
pub enum Essence {
    /// A regular transaction essence.
    #[packable(tag = RegularEssence::KIND)]
    Regular(RegularEssence),
}

impl Essence {
    /// Returns the essence kind of an `Essence`.
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => RegularEssence::KIND,
        }
    }

    /// Return the Blake2b hash of an `Essence`.
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_to_vec().unwrap()).into()
    }
}

impl From<RegularEssence> for Essence {
    fn from(essence: RegularEssence) -> Self {
        Self::Regular(essence)
    }
}
