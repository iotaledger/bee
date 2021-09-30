// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageMetadata};

use std::sync::Arc;

/// Data structure used by the [`Tangle`](crate::Tangle) to store a [`Message`] and its associated [`MessageMetadata`].
#[derive(Clone, Debug)]
pub struct MessageData {
    message: Arc<Message>,
    metadata: MessageMetadata,
}

impl MessageData {
    pub(crate) fn new(message: Message, metadata: MessageMetadata) -> MessageData {
        MessageData {
            message: Arc::new(message),
            metadata,
        }
    }

    /// Returns the [`Message`] of the [`MessageData`].
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Returns the [`MessageMetadata`] of the [`MessageData`].
    pub fn metadata(&self) -> &MessageMetadata {
        &self.metadata
    }
}
