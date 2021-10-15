// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

///
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpirationMilestoneIndexFeatureBlock {}

impl ExpirationMilestoneIndexFeatureBlock {
    /// The feature block kind of an `ExpirationMilestoneIndexFeatureBlock`.
    pub const KIND: u8 = 5;

    /// Creates a new `ExpirationMilestoneIndexFeatureBlock`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Packable for ExpirationMilestoneIndexFeatureBlock {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0
    }

    fn pack<W: Write>(&self, _writer: &mut W) -> Result<(), Self::Error> {
        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(_reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new())
    }
}
