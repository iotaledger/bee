// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};

use parking_lot::RwLock;

use std::{collections::HashMap, sync::Arc};

/// Data structure used by the [`Tangle`](crate::Tangle) to store a [`Message`] and its associated [`MessageMetadata`].
#[derive(Clone)]
pub struct MessageData {
    message: Arc<Message>,
    metadata: MessageMetadata,
}

/// Tangle data structure, providing access to [`Message`]s and [`MessageMetadata`]s.
#[derive(Default)]
pub struct Tangle(RwLock<HashMap<MessageId, MessageData>>);

impl Tangle {
    /// Creates a new [`Tangle`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`]/[`MessageMetadata`] pair, associated with a [`MessageId`], into the [`Tangle`].
    pub fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.0.write().insert(
            message_id,
            MessageData {
                message: Arc::new(message),
                metadata,
            },
        );
    }

    /// Retrieves a [`Message`]/[`MessageMetadata`] pair, associated with a [`MessageId`], from the [`Tangle`].
    pub fn get(&self, message_id: &MessageId) -> Option<(Arc<Message>, MessageMetadata)> {
        self.0
            .read()
            .get(message_id)
            .map(|data| (data.message.clone(), data.metadata.clone()))
    }
}
