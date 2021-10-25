// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::milestone::MilestoneIndex;
use bee_packable::Packable;

use core::ops::Deref;

/// A wrapper type to represent the current ledger index.
#[derive(Debug, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Packable)]
pub struct LedgerIndex(pub MilestoneIndex);

impl LedgerIndex {
    /// Creates a new `LedgerIndex`.
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }
}

impl From<MilestoneIndex> for LedgerIndex {
    fn from(index: MilestoneIndex) -> Self {
        Self(index)
    }
}

impl Deref for LedgerIndex {
    type Target = <MilestoneIndex as Deref>::Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
