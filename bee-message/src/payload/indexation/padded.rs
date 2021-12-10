// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// An indexation payload index padded with `0` up to the maximum length.
#[derive(Clone, Copy, Eq, PartialEq, Hash, derive_more::From)]
pub struct PaddedIndex([u8; PaddedIndex::LENGTH]);

impl PaddedIndex {
    /// Length of a [`PaddedIndex`].
    pub const LENGTH: usize = 64;

    /// Creates a new [`PaddedIndex`].
    pub fn new(bytes: [u8; PaddedIndex::LENGTH]) -> Self {
        bytes.into()
    }
}

impl AsRef<[u8]> for PaddedIndex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl std::ops::Deref for PaddedIndex {
    type Target = [u8; PaddedIndex::LENGTH];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for PaddedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for PaddedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "PaddedIndex({})", self)
    }
}
