// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Length of an indexation padded index.
pub const INDEXATION_PADDED_INDEX_LENGTH: usize = 64;

/// An indexation payload index padded with `0` up to the maximum length.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct PaddedIndex([u8; INDEXATION_PADDED_INDEX_LENGTH]);

impl PaddedIndex {
    /// Creates a new `PaddedIndex`.
    pub fn new(bytes: [u8; INDEXATION_PADDED_INDEX_LENGTH]) -> Self {
        bytes.into()
    }
}

impl From<[u8; INDEXATION_PADDED_INDEX_LENGTH]> for PaddedIndex {
    fn from(bytes: [u8; INDEXATION_PADDED_INDEX_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for PaddedIndex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl std::ops::Deref for PaddedIndex {
    type Target = [u8; INDEXATION_PADDED_INDEX_LENGTH];

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
