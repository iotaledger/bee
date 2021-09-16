// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{Message, MessageId, MessageMetadata};

use tokio::sync::Mutex;

use std::{collections::HashMap, sync::Arc};

struct MessageData {
    message: Arc<Message>,
    metadata: MessageMetadata,
}

/// Tangle data structure.
/// Provides a [`HashMap`] of [`MessageId`]s to [`Message`]s.
#[derive(Default)]
pub struct Tangle(Mutex<HashMap<MessageId, MessageData>>);

impl Tangle {
    /// Creates a new tangle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`] into the tangle, associating it with a [`MessageId`].
    pub async fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.0.lock().await.insert(
            message_id,
            MessageData {
                message: Arc::new(message),
                metadata,
            },
        );
    }

    /// Retrieves a [`Message`] from the tangle with a given [`MessageId`].
    pub async fn get(&self, message_id: &MessageId) -> Option<Arc<Message>> {
        self.0.lock().await.get(message_id).map(|data| data.message.clone())
    }

    /// Retrieves [`MessageMetadata`] from the tangle with a given [`MessageId`].
    pub async fn get_metadata(&self, message_id: &MessageId) -> Option<MessageMetadata> {
        self.0.lock().await.get(message_id).map(|data| data.metadata.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::Tangle;

    use bee_test::rand::message::{metadata::rand_message_metadata, rand_message};

    #[tokio::test]
    async fn test_insert() {
        let message = rand_message();
        let message_id = message.id();
        let metadata = rand_message_metadata();

        let tangle = Tangle::new();
        tangle.insert(message_id, message.clone(), metadata.clone()).await;

        assert_eq!(*tangle.get(&message_id).await.unwrap(), message);
        assert_eq!(tangle.get_metadata(&message_id).await.unwrap(), metadata);
    }
}
