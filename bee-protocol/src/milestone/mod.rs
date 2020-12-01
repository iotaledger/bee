// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod index;
pub(crate) mod key_manager;
pub(crate) mod key_range;

pub use index::MilestoneIndex;

use bee_message::MessageId;

#[derive(Clone)]
pub struct Milestone {
    pub(crate) index: MilestoneIndex,
    pub(crate) message_id: MessageId,
}

impl Milestone {
    pub fn new(index: MilestoneIndex, message_id: MessageId) -> Self {
        Self { index, message_id }
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}
