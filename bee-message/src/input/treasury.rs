// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{payload::milestone::MilestoneId, Error};

use bee_common::packable::{Packable, Read, Write};

use core::{ops::Deref, str::FromStr};

/// `TreasuryInput` is an input which references a milestone which generated a `TreasuryOutput`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, derive_more::From)]
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

impl Deref for TreasuryInput {
    type Target = MilestoneId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

impl Packable for TreasuryInput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(MilestoneId::unpack_inner::<R, CHECK>(reader)?))
    }
}
