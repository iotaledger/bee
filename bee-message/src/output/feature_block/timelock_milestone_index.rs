// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use bee_common::packable::{Packable, Read, Write};

/// Defines a milestone index until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockMilestoneIndexFeatureBlock {
    // The milestone index starting from which the output can be consumed.
    index: MilestoneIndex,
}

impl TimelockMilestoneIndexFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of a [`TimelockMilestoneIndexFeatureBlock`].
    pub const KIND: u8 = 3;

    /// Creates a new [`TimelockMilestoneIndexFeatureBlock`].
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }

    /// Returns the index.
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}

impl Packable for TimelockMilestoneIndexFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self {
            index: MilestoneIndex::unpack_inner::<R, CHECK>(reader)?,
        })
    }
}
