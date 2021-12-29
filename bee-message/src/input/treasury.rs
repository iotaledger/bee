// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::milestone::MilestoneId, Error};

use derive_more::{Deref, From};

use core::str::FromStr;

/// `TreasuryInput` is an input which references a milestone which generated a `TreasuryOutput`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, From, Deref, bee_packable::Packable)]
pub struct TreasuryInput(MilestoneId);

impl TreasuryInput {
    /// The input kind of a `TreasuryInput`.
    pub const KIND: u8 = 1;

    /// Creates a new `TreasuryInput`.
    pub fn new(id: MilestoneId) -> Self {
        Self(id)
    }

    /// Returns the milestones id of a `TreasuryInput`.
    pub fn milestone_id(&self) -> &MilestoneId {
        &self.0
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(TreasuryInput);

impl FromStr for TreasuryInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TreasuryInput(MilestoneId::from_str(s)?))
    }
}

impl core::fmt::Display for TreasuryInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for TreasuryInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TreasuryInput({})", self.0)
    }
}
