// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use bee_common::packable::{Packable, Read, Write};

use derive_more::From;

/// Defines a milestone index until which only the deposit [`Address`](crate::address::Address) is allowed to unlock the
/// output. After the milestone index, only the [`Address`](crate::address::Address) defined in the
/// [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationMilestoneIndexFeatureBlock(
    // Before this milestone index, [`Address`](crate::address::Address) is allowed to unlock the output.
    // After that, only the [`Address`](crate::address::Address) defined in the
    // [`SenderFeatureBlock`](crate::output::feature_block::SenderFeatureBlock) can.
    MilestoneIndex,
);

impl ExpirationMilestoneIndexFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of an [`ExpirationMilestoneIndexFeatureBlock`].
    pub const KIND: u8 = 5;

    /// Creates a new [`ExpirationMilestoneIndexFeatureBlock`].
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

impl Packable for ExpirationMilestoneIndexFeatureBlock {
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
