// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bee_block::{Block, BlockId};

use crate::{metadata::BlockMetadata, MessageRef, VecSet};

#[derive(Clone)]
pub struct Vertex {
    message: Option<(MessageRef, BlockMetadata)>,
    children: (VecSet<BlockId>, bool), // Exhaustive flag
    eviction_blocks: isize,
}

impl Vertex {
    pub fn empty() -> Self {
        Self {
            message: None,
            children: (VecSet::default(), false),
            eviction_blocks: 0,
        }
    }

    pub fn new(message: Message, metadata: BlockMetadata) -> Self {
        Self {
            message: Some((MessageRef(Arc::new(message)), metadata)),
            children: (VecSet::default(), false),
            eviction_blocks: 0,
        }
    }

    pub fn parents(&self) -> Option<impl Iterator<Item = &BlockId> + '_> {
        self.message().map(|m| m.parents().iter())
    }

    pub fn message_and_metadata(&self) -> Option<&(MessageRef, BlockMetadata)> {
        self.message.as_ref()
    }

    pub fn message(&self) -> Option<&MessageRef> {
        self.message_and_metadata().map(|(m, _)| m)
    }

    pub fn metadata(&self) -> Option<&BlockMetadata> {
        self.message_and_metadata().map(|(_, m)| m)
    }

    pub fn metadata_mut(&mut self) -> Option<&mut BlockMetadata> {
        self.message.as_mut().map(|(_, m)| m)
    }

    pub fn add_child(&mut self, child: BlockId) {
        self.children.0.insert(child);
    }

    pub fn children(&self) -> &[BlockId] {
        &self.children.0
    }

    pub fn children_exhaustive(&self) -> bool {
        self.children.1
    }

    /// Set the exhaustive flag. This should not be done if the vertex's children are exhaustive.
    pub(crate) fn set_exhaustive(&mut self) {
        self.children.1 = true;
    }

    pub(crate) fn insert_message_and_metadata(&mut self, message: Message, metadata: BlockMetadata) {
        self.message = Some((MessageRef(Arc::new(message)), metadata));
    }

    pub(crate) fn prevent_eviction(&mut self) {
        self.eviction_blocks += 1;
    }

    pub(crate) fn allow_eviction(&mut self) {
        self.eviction_blocks -= 1;
        assert!(self.eviction_blocks >= 0);
    }

    pub(crate) fn can_evict(&self) -> bool {
        self.eviction_blocks == 0
    }
}
