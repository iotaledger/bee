// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_packable::Packable;

use std::ops::Deref;

/// A type representing an unreferenced message.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Packable)]
pub struct UnreferencedMessage(MessageId);

impl From<MessageId> for UnreferencedMessage {
    fn from(message_id: MessageId) -> Self {
        Self(message_id)
    }
}

impl Deref for UnreferencedMessage {
    type Target = MessageId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnreferencedMessage {
    /// Create a new `UnreferencedMessage`.
    pub fn new(message_id: MessageId) -> Self {
        message_id.into()
    }

    /// Get the message ID of this unreferenced message.
    pub fn message_id(&self) -> &MessageId {
        &self.0
    }
}
