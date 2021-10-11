// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{metadata::MessageMetadata, MessageRef, VecSet};

use bee_message::{Message, MessageId};

use std::sync::Arc;

#[derive(Clone)]
pub struct Vertex {
    message: Option<(MessageRef, MessageMetadata)>,
    children: (VecSet<MessageId>, bool), // Exhaustive flag
}

impl Vertex {
    pub fn empty() -> Self {
        Self {
            message: None,
            children: (VecSet::default(), false),
        }
    }

    pub fn new(message: Message, metadata: MessageMetadata) -> Self {
        Self {
            message: Some((MessageRef(Arc::new(message)), metadata)),
            children: (VecSet::default(), false),
        }
    }

    pub fn parents(&self) -> Option<impl Iterator<Item = &MessageId> + '_> {
        self.message().map(|m| m.parents().iter())
    }

    pub fn message_and_metadata(&self) -> Option<&(MessageRef, MessageMetadata)> {
        self.message.as_ref()
    }

    pub fn message(&self) -> Option<&MessageRef> {
        self.message_and_metadata().map(|(m, _)| m)
    }

    pub fn metadata(&self) -> Option<&MessageMetadata> {
        self.message_and_metadata().map(|(_, m)| m)
    }

    pub fn metadata_mut(&mut self) -> Option<&mut MessageMetadata> {
        self.message.as_mut().map(|(_, m)| m)
    }

    pub fn add_child(&mut self, child: MessageId) {
        self.children.0.insert(child);
    }

    pub fn children(&self) -> &[MessageId] {
        &self.children.0
    }

    pub fn children_exhaustive(&self) -> bool {
        self.children.1
    }

    /// Set the exhaustive flag. This should not be done if the vertex's children are exhaustive.
    pub(crate) fn set_exhaustive(&mut self) {
        self.children.1 = true;
    }

    pub(crate) fn insert_message_and_metadata(&mut self, message: Message, metadata: MessageMetadata) {
        self.message = Some((MessageRef(Arc::new(message)), metadata));
    }
}
