// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::MessageData;

use bee_message::{Message, MessageId, MessageMetadata};

use tokio::sync::RwLock;

use std::{collections::HashMap, sync::Arc};

/// Tangle data structure, providing access to [`Message`]s and [`MessageMetadata`]s.
#[derive(Default)]
pub struct Tangle(RwLock<HashMap<MessageId, Arc<MessageData>>>);

impl Tangle {
    /// Creates a new [`Tangle`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`]/[`MessageMetadata`] pair, associated with a [`MessageId`], into the [`Tangle`].
    pub async fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.0
            .write()
            .await
            .insert(message_id, Arc::new(MessageData::new(message, metadata)));
    }

    /// Retrieves the [`MessageData`], associated with a [`MessageId`], from the [`Tangle`] .
    pub async fn get(&self, message_id: &MessageId) -> Option<Arc<MessageData>> {
        self.0.read().await.get(message_id).cloned()
    }
}
