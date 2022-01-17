// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod regular;

pub(crate) use regular::{InputCount, OutputCount};
pub use regular::{RegularTransactionEssence, RegularTransactionEssenceBuilder};

use crate::Error;

use crypto::hashes::{blake2b::Blake2b256, Digest};
use derive_more::From;
use packable::PackableExt;

/// A generic essence that can represent different types defining transaction essences.
#[derive(Clone, Debug, Eq, PartialEq, From, packable::Packable)]
#[cfg_attr(
    feature = "serde1",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type", content = "data")
)]
#[packable(unpack_error = Error)]
#[packable(tag_type = u8, with_error = Error::InvalidEssenceKind)]
pub enum TransactionEssence {
    /// A regular transaction essence.
    #[packable(tag = RegularTransactionEssence::KIND)]
    Regular(RegularTransactionEssence),
}

impl TransactionEssence {
    /// Returns the essence kind of an [`TransactionEssence`].
    pub fn kind(&self) -> u8 {
        match self {
            Self::Regular(_) => RegularTransactionEssence::KIND,
        }
    }

    /// Return the Blake2b hash of an [`TransactionEssence`].
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_to_vec()).into()
    }
}
