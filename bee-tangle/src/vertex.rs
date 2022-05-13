// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bee_block::{Block, BlockId};

use crate::{metadata::BlockMetadata, BlockRef, VecSet};

#[derive(Clone)]
pub struct Vertex {
    block: Option<(BlockRef, BlockMetadata)>,
    children: (VecSet<BlockId>, bool), // Exhaustive flag
    eviction_blocks: isize,
}

impl Vertex {
    pub fn empty() -> Self {
        Self {
            block: None,
            children: (VecSet::default(), false),
            eviction_blocks: 0,
        }
    }

    pub fn new(block: Block, metadata: BlockMetadata) -> Self {
        Self {
            block: Some((BlockRef(Arc::new(block)), metadata)),
            children: (VecSet::default(), false),
            eviction_blocks: 0,
        }
    }

    pub fn parents(&self) -> Option<impl Iterator<Item = &BlockId> + '_> {
        self.block().map(|m| m.parents().iter())
    }

    pub fn block_and_metadata(&self) -> Option<&(BlockRef, BlockMetadata)> {
        self.block.as_ref()
    }

    pub fn block(&self) -> Option<&BlockRef> {
        self.block_and_metadata().map(|(m, _)| m)
    }

    pub fn metadata(&self) -> Option<&BlockMetadata> {
        self.block_and_metadata().map(|(_, m)| m)
    }

    pub fn metadata_mut(&mut self) -> Option<&mut BlockMetadata> {
        self.block.as_mut().map(|(_, m)| m)
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

    pub(crate) fn insert_block_and_metadata(&mut self, block: Block, metadata: BlockMetadata) {
        self.block = Some((BlockRef(Arc::new(block)), metadata));
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
