// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use bee_common::packable::{Packable, Read, Write};

/// Defines a milestone index until which only the deposit [`Address`] is allowed to unlock the output.
/// After the milestone index, only the [`Address`] defined in the [`SenderFeatureBlock`] can unlock it.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, derive_more::From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationMilestoneIndexFeatureBlock {
    // Before this milestone index, [`Address`] is allowed to unlock the output.
    // After that, only the [`Address`] defined in the [`SenderFeatureBlock`] can.
    index: MilestoneIndex,
}

impl ExpirationMilestoneIndexFeatureBlock {
    /// The [`FeatureBlock`] kind of an [`ExpirationMilestoneIndexFeatureBlock`].
    pub const KIND: u8 = 5;

    /// Creates a new [`ExpirationMilestoneIndexFeatureBlock`].
    pub fn new(index: MilestoneIndex) -> Self {
        index.into()
    }

    /// Returns the index.
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}

impl Packable for ExpirationMilestoneIndexFeatureBlock {
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
