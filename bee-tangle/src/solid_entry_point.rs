// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A SolidEntryPoint is a [`BlockId`](bee_block::BlockId) of a message that is solid even if we do not have them
//! or their past in the database. They often come from a snapshot file and allow a node to solidify
//! without needing the full tangle history.

use core::{convert::AsRef, ops::Deref};

use bee_block::BlockId;
use ref_cast::RefCast;

/// A SolidEntryPoint is a [`BlockId`] of a message that is solid even if we do not have them
/// or their past in the database. They often come from a snapshot file and allow a node to solidify
/// without needing the full tangle history.
#[derive(RefCast)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, packable::Packable)]
pub struct SolidEntryPoint(BlockId);

impl SolidEntryPoint {
    /// Create a `SolidEntryPoint` from an existing `BlockId`.
    pub fn new(message_id: BlockId) -> Self {
        message_id.into()
    }

    /// Create a null `SolidEntryPoint` (the zero-message).
    pub fn null() -> Self {
        Self(BlockId::null())
    }

    /// Returns the underlying `BlockId`.
    pub fn message_id(&self) -> &BlockId {
        &self.0
    }
}

impl From<BlockId> for SolidEntryPoint {
    fn from(message_id: BlockId) -> Self {
        Self(message_id)
    }
}

impl AsRef<BlockId> for SolidEntryPoint {
    fn as_ref(&self) -> &BlockId {
        &self.0
    }
}

impl Deref for SolidEntryPoint {
    type Target = BlockId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
