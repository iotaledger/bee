// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;

use std::ops::Deref;

pub struct SolidEntryPoint(MessageId);

impl From<MessageId> for SolidEntryPoint {
    fn from(message_id: MessageId) -> Self {
        Self(message_id)
    }
}

impl Deref for SolidEntryPoint {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SolidEntryPoint {
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}
