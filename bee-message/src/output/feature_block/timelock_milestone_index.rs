// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockMilestoneIndexFeatureBlock(MilestoneIndex);

impl TimelockMilestoneIndexFeatureBlock {
    /// The feature block kind of a `TimelockMilestoneIndexFeatureBlock`.
    pub const KIND: u8 = 3;

    /// Creates a new `TimelockMilestoneIndexFeatureBlock`.
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }

    /// Returns the index.
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
        Ok(Self::new(MilestoneIndex::unpack_inner::<R, CHECK>(reader)?))
    }
}
