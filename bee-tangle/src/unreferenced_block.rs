// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;

use bee_block::BlockId;

/// A type representing an unreferenced block.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, packable::Packable)]
pub struct UnreferencedBlock(BlockId);

impl From<BlockId> for UnreferencedBlock {
    fn from(message_id: BlockId) -> Self {
        Self(message_id)
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
    pub fn new(message_id: BlockId) -> Self {
        message_id.into()
    }

    /// Get the message ID of this unreferenced message.
    pub fn message_id(&self) -> &BlockId {
        &self.0
    }
}
