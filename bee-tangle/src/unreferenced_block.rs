// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;

use bee_block::BlockId;

/// A type representing an unreferenced block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, packable::Packable)]
pub struct UnreferencedBlock(BlockId);

impl From<BlockId> for UnreferencedBlock {
    fn from(block_id: BlockId) -> Self {
        Self(block_id)
    }
}

impl Deref for UnreferencedBlock {
    type Target = BlockId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnreferencedBlock {
    /// Create a new [`UnreferencedBlock`].
    pub fn new(block_id: BlockId) -> Self {
        block_id.into()
    }

    /// Get the block ID of this unreferenced block.
    pub fn block_id(&self) -> &BlockId {
        &self.0
    }
}
