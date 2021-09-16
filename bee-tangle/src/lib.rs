use bee_message::{Message, MessageId};

use tokio::sync::Mutex;

use std::{collections::HashMap, sync::Arc};

/// Tangle data structure.
/// Provides a [`HashMap`] of [`MessageId`]s to [`Message`]s.
#[derive(Default)]
pub struct Tangle(Mutex<HashMap<MessageId, Arc<Message>>>);

impl Tangle {
    /// Creates a new tangle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a [`Message`] into the tangle, associating it with a [`MessageId`].
    pub async fn insert(&self, msg_id: MessageId, msg: Message) {
        self.0.lock().await.insert(msg_id, Arc::new(msg));
    }

    /// Retrieves a [`Message`] from the tangle with a given [`MessageId`].
    pub async fn get(&self, msg_id: &MessageId) -> Option<Arc<Message>> {
        self.0.lock().await.get(msg_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::Tangle;

    use bee_test::rand::message::rand_message;

    #[tokio::test]
    async fn test_insert() {
        let message = rand_message();
        let message_id = message.id();

        let tangle = Tangle::new();
        tangle.insert(message_id, message.clone()).await;

        assert_eq!(*tangle.get(&message_id).await.unwrap(), message);
    }
}
