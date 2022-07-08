// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use bee_block::{output::Output, payload::milestone::MilestoneIndex, BlockId};
use packable::{error::UnpackError, error::UnpackErrorExt, unpacker::Unpacker, Packable};

use crate::types::error::Error;

/// Represents a newly created output.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[packable(unpack_error = Error)]
pub struct CreatedOutput {
    block_id: BlockId,
    milestone_index: MilestoneIndex,
    milestone_timestamp: u32,
    inner: Output,
}

impl CreatedOutput {
    /// Creates a new [`CreatedOutput`].
    pub fn new(block_id: BlockId, milestone_index: MilestoneIndex, milestone_timestamp: u32, inner: Output) -> Self {
        Self {
            block_id,
            milestone_index,
            milestone_timestamp,
            inner,
        }
    }

    /// Returns the block id of the [`CreatedOutput`].
    pub fn block_id(&self) -> &BlockId {
        &self.block_id
    }

    /// Returns the milestone index of the [`CreatedOutput`].
    pub fn milestone_index(&self) -> MilestoneIndex {
        self.milestone_index
    }

    /// Returns the milestone milestone timestamp of the [`CreatedOutput`].
    pub fn milestone_timestamp(&self) -> u32 {
        self.milestone_timestamp
    }

    /// Returns the inner output of the [`CreatedOutput`].
    pub fn inner(&self) -> &Output {
        &self.inner
    }

    /// This method exists only because Hornet needs a length prefix for the inner output.
    /// As it diverges from the usual serialization, a specific method is required.
    pub(crate) fn unpack_with_length<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<<Self as Packable>::UnpackError, U::Error>> {
        let block_id = BlockId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone_timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let _ = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let inner = Output::unpack::<_, VERIFY>(unpacker).coerce()?;

        Ok(Self {
            block_id,
            milestone_index,
            milestone_timestamp,
            inner,
        })
    }
}

impl Deref for CreatedOutput {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
