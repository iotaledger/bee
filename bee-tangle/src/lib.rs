// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};

use tokio::sync::RwLock;

use std::{collections::HashMap, sync::Arc};

/// Data structure used by the [`Tangle`] to store a [`Message`] with its associated [`MessageMetadata`].
pub struct MessageData {
    message: Message,
    metadata: MessageMetadata,
}

impl MessageData {
    /// Retrieves the [`Message`] from the [`MessageData`].
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Retrieves the [`MessageMetadata`] from the [`MessageData`].
    pub fn metadata(&self) -> &MessageMetadata {
        &self.metadata
    }
}

/// Tangle data structure.
/// Provides a [`HashMap`] of [`MessageId`]s to [`MessageData`]s..
#[derive(Default)]
pub struct Tangle(RwLock<HashMap<MessageId, Arc<MessageData>>>);

impl Tangle {
    /// Creates a new [`Tangle`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`] and its [`MessageMetadata`] into the [`Tangle`], associating it with a [`MessageId`].
    pub async fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.0
            .write()
            .await
            .insert(message_id, Arc::new(MessageData { message, metadata }));
    }

    /// Retrieves the [`MessageData`] from the [`Tangle`] with a given [`MessageId`].
    pub async fn get(&self, message_id: &MessageId) -> Option<Arc<MessageData>> {
        self.0.read().await.get(message_id).cloned()
    }
}
