// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::error::Error;

use bee_message::{milestone::MilestoneIndex, output::Output, MessageId};

use core::ops::Deref;

/// Represents a newly created output.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
#[packable(unpack_error = Error)]
pub struct CreatedOutput {
    message_id: MessageId,
    milestone_index: MilestoneIndex,
    milestone_timestamp: u32,
    inner: Output,
}

impl CreatedOutput {
    /// Creates a new [`CreatedOutput`].
    pub fn new(
        message_id: MessageId,
        milestone_index: MilestoneIndex,
        milestone_timestamp: u32,
        inner: Output,
    ) -> Self {
        Self {
            message_id,
            milestone_index,
            milestone_timestamp,
            inner,
        }
    }

    /// Returns the message id of the [`CreatedOutput`].
    pub fn message_id(&self) -> &MessageId {
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
