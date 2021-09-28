// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageMetadata};

/// Data structure used by the [`Tangle`] to store a [`Message`] and its associated [`MessageMetadata`].
pub struct MessageData {
    message: Message,
    metadata: MessageMetadata,
}

impl MessageData {
    /// Creates a new [`MessageData`].
    pub fn new(message: Message, metadata: MessageMetadata) -> Self {
        Self { message, metadata }
    }

    /// Returns the [`Message`] of the [`MessageData`].
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Return the [`MessageMetadata`] of the [`MessageData`].
    pub fn metadata(&self) -> &MessageMetadata {
        &self.metadata
    }
}
