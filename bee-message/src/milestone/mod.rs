// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageId;

/// Defines a coordinator milestone.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct Milestone {
    message_id: MessageId,
    timestamp: u32,
}

impl Milestone {
    /// Creates a new `Milestone`.
    pub fn new(message_id: MessageId, timestamp: u32) -> Self {
        Self { message_id, timestamp }
    }

    /// Returns the message id of a `Milestone`.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the timestamp of a `Milestone`.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }
}
