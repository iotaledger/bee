// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::milestone::MilestoneId, MessageId};

/// Defines milestone metadata.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct MilestoneMetadata {
    message_id: MessageId,
    milestone_id: MilestoneId,
    timestamp: u32,
}

impl MilestoneMetadata {
    /// Creates a new [`MilestoneMetadata`].
    pub fn new(message_id: MessageId, milestone_id: MilestoneId, timestamp: u32) -> Self {
        Self {
            message_id,
            milestone_id,
            timestamp,
        }
    }

    /// Returns the message id of a [`MilestoneMetadata`].
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the milestone id of a [`MilestoneMetadata`].
    pub fn milestone_id(&self) -> &MilestoneId {
        &self.milestone_id
    }

    /// Returns the timestamp of a [`MilestoneMetadata`].
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}
