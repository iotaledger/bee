// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};

use tokio::sync::RwLock;

use std::{collections::HashMap, sync::Arc};

struct MessageData {
    message: Arc<Message>,
    metadata: MessageMetadata,
}

/// Tangle data structure.
/// Provides a [`HashMap`] of [`MessageId`]s to [`Message`]s.
#[derive(Default)]
pub struct Tangle(RwLock<HashMap<MessageId, MessageData>>);

impl Tangle {
    /// Creates a new [`Tangle`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`] and its [`MessageMetadata`] into the [`Tangle`], associating it with a [`MessageId`].
    pub async fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.0.write().await.insert(
            message_id,
            MessageData {
                message: Arc::new(message),
                metadata,
            },
        );
    }

    /// Retrieves a [`Message`] from the [`Tangle`] with a given [`MessageId`].
    pub async fn get_message(&self, message_id: &MessageId) -> Option<Arc<Message>> {
        self.0.read().await.get(message_id).map(|data| data.message.clone())
    }

    /// Retrieves [`MessageMetadata`] from the [`Tangle`] with a given [`MessageId`].
    pub async fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.0.read().await.get(message_id).map(|data| data.metadata.clone())
    }
}
