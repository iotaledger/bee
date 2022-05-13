// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::ops::Deref;

use bee_block::{output::Output, payload::milestone::MilestoneIndex, BlockId};

use crate::types::error::Error;

/// Represents a newly created output.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[packable(unpack_error = Error)]
pub struct CreatedOutput {
    message_id: BlockId,
    milestone_index: MilestoneIndex,
    milestone_timestamp: u32,
    inner: Output,
}

impl CreatedOutput {
    /// Creates a new [`CreatedOutput`].
    pub fn new(message_id: BlockId, milestone_index: MilestoneIndex, milestone_timestamp: u32, inner: Output) -> Self {
        Self {
            message_id,
            milestone_index,
            milestone_timestamp,
            inner,
        }
    }

    /// Returns the message id of the [`CreatedOutput`].
    pub fn message_id(&self) -> &BlockId {
        &self.message_id
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
}

impl Deref for CreatedOutput {
    type Target = Output;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
