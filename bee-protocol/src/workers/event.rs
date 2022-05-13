// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::BlockId;

/// An event that indicates that a block was processed.
#[derive(Clone)]
pub struct BlockProcessed {
    /// Block identifier of the processed block.
    pub block_id: BlockId,
}

/// An event that indicates that a block was solidified.
#[derive(Clone)]
pub struct BlockSolidified {
    /// Block identifier of the solidified block.
    pub block_id: BlockId,
}

/// An event that indicates that the MPS metrics were updated.
#[derive(Clone)]
pub struct MpsMetricsUpdated {
    /// Number of incoming blocks.
    pub incoming: u64,
    /// Number of new blocks.
    pub new: u64,
    /// Number of known blocks.
    pub known: u64,
    /// Number of invalid blocks.
    pub invalid: u64,
    /// Number of outgoing blocks.
    pub outgoing: u64,
}

/// An event that indicates that a vertex was created.
#[derive(Clone)]
pub struct VertexCreated {
    /// Block identifier of the created vertex.
    pub block_id: BlockId,
    /// Block identifiers of the parents of the created vertex.
    pub parent_block_ids: Vec<BlockId>,
    /// Whether the vertex is a solid block.
    pub is_solid: bool,
    /// Whether the vertex is a referenced block.
    pub is_referenced: bool,
    /// Whether the vertex is a conflicting block.
    pub is_conflicting: bool,
    /// Whether the vertex is a milestone block.
    pub is_milestone: bool,
    /// Whether the vertex is a tip block.
    pub is_tip: bool,
    /// Whether the vertex is a selected block.
    pub is_selected: bool,
}

/// An event that indicates that a tip was added.
#[derive(Clone)]
pub struct TipAdded {
    /// Block identifier of the added tip.
    pub block_id: BlockId,
}

/// An event that indicates that a tip was removed.
#[derive(Clone)]
pub struct TipRemoved {
    /// Block identifier of the removed tip.
    pub block_id: BlockId,
}
