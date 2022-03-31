// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod index;

pub use self::index::MilestoneIndex;
use crate::MessageId;

/// Defines a coordinator milestone.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct Milestone {
    message_id: MessageId,
    timestamp: u64,
}

impl Milestone {
    /// Creates a new `Milestone`.
    pub fn new(message_id: MessageId, timestamp: u64) -> Self {
        Self { message_id, timestamp }
    }

    /// Returns the message id of a `Milestone`.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the timestamp of a `Milestone`.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
