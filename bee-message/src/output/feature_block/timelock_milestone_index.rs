// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use bee_common::packable::{Packable, Read, Write};

use derive_more::From;

/// Defines a milestone index until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockMilestoneIndexFeatureBlock(
    // The milestone index starting from which the output can be consumed.
    MilestoneIndex,
);

impl TimelockMilestoneIndexFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockMilestoneIndexFeatureBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`TimelockMilestoneIndexFeatureBlock`].
    #[inline(always)]
    pub fn new(index: MilestoneIndex) -> Self {
        Self(index)
    }

    /// Returns the index.
    #[inline(always)]
    pub fn index(&self) -> MilestoneIndex {
        self.0
    }
}

impl Packable for TimelockMilestoneIndexFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(MilestoneIndex::unpack_inner::<R, CHECK>(reader)?))
    }
}
