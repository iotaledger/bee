// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{MessageData, StorageBackend, TangleConfig};

use bee_message::{Message, MessageId, MessageMetadata};

use parking_lot::RwLock;

use std::collections::HashMap;

/// Tangle data structure, providing access to [`Message`]s and [`MessageMetadata`]s.
#[derive(Default)]
pub struct Tangle<T> {
    _config: TangleConfig,
    cache: RwLock<HashMap<MessageId, MessageData>>,
    _storage: T,
}

impl<T: StorageBackend> Tangle<T> {
    /// Creates a new [`Tangle`].
    pub fn new(config: TangleConfig, storage: T) -> Self {
        Self {
            _config: config,
            cache: Default::default(),
            _storage: storage,
        }
    }

    /// Inserts a [`Message`]/[`MessageMetadata`] pair, associated with a [`MessageId`], into the [`Tangle`].
    pub fn insert(&self, message_id: MessageId, message: Message, metadata: MessageMetadata) {
        self.cache
            .write()
            .insert(message_id, MessageData::new(message, metadata));
    }

    /// Retrieves a [`Message`]/[`MessageMetadata`] pair, associated with a [`MessageId`], from the [`Tangle`].
    pub fn get(&self, message_id: &MessageId) -> Option<MessageData> {
        self.cache.read().get(message_id).cloned()
    }
}
