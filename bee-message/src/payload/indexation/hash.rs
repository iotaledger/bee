// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub const HASHED_INDEX_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct HashedIndex([u8; HASHED_INDEX_LENGTH]);

impl HashedIndex {
    pub fn new(bytes: [u8; HASHED_INDEX_LENGTH]) -> Self {
        bytes.into()
    }
}

impl From<[u8; HASHED_INDEX_LENGTH]> for HashedIndex {
    fn from(bytes: [u8; HASHED_INDEX_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for HashedIndex {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl core::fmt::Display for HashedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for HashedIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "HashedIndex({})", self)
    }
}
