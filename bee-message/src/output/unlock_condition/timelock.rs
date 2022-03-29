// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{milestone::MilestoneIndex, Error};

use derive_more::From;
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

/// Defines a milestone index and/or unix timestamp until which the output can not be unlocked.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct TimelockUnlockCondition {
    // The milestone index starting from which the output can be consumed.
    milestone_index: MilestoneIndex,
    // Unix time, seconds since unix epoch, starting from which the output can be consumed.
    timestamp: u32,
}

impl TimelockUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of a [`TimelockUnlockCondition`].
    pub const KIND: u8 = 2;

    /// Creates a new [`TimelockUnlockCondition`].
    #[inline(always)]
    pub fn new(milestone_index: MilestoneIndex, timestamp: u32) -> Result<Self, Error> {
        verify_milestone_index_timestamp(milestone_index, timestamp)?;

        Ok(Self {
            milestone_index,
            timestamp,
        })
    }

    /// Returns the milestone index of a [`TimelockUnlockCondition`].
    #[inline(always)]
    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    /// Returns the timestamp of a [`TimelockUnlockCondition`].
    #[inline(always)]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}

impl Packable for TimelockUnlockCondition {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.milestone_index.pack(packer)?;
        self.timestamp.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;
        let timestamp = u32::unpack::<_, VERIFY>(unpacker).infallible()?;

        if VERIFY {
            verify_milestone_index_timestamp(milestone_index, timestamp).map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            milestone_index,
            timestamp,
        })
    }
}

#[inline]
fn verify_milestone_index_timestamp(milestone_index: MilestoneIndex, timestamp: u32) -> Result<(), Error> {
    if *milestone_index == 0 && timestamp == 0 {
        Err(Error::TimelockUnlockConditionZero)
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::{
        dto::{is_zero, is_zero_milestone},
        milestone::MilestoneIndex,
    };

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TimelockUnlockConditionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        #[serde(rename = "milestoneIndex")]
        #[serde(skip_serializing_if = "is_zero_milestone", default)]
        pub milestone_index: MilestoneIndex,
        #[serde(rename = "unixTime")]
        #[serde(skip_serializing_if = "is_zero", default)]
        pub timestamp: u32,
    }
}
