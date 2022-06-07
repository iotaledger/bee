// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the event occurring during ledger operations.

use std::collections::HashMap;

use bee_block::{
    output::{Output, OutputId},
    payload::milestone::MilestoneIndex,
    semantic::ConflictReason,
    BlockId,
};

use crate::types::{ConsumedOutput, CreatedOutput, Receipt};

/// An event that indicates that a milestone was confirmed.
#[derive(Clone)]
pub struct MilestoneConfirmed {
    /// The block identifier of the milestone.
    pub block_id: BlockId,
    /// The index of the milestone.
    pub index: MilestoneIndex,
    /// The timestamp of the milestone.
    pub timestamp: u32,
    /// The number of blocks referenced by the milestone.
    pub referenced_blocks: usize,
    /// The blocks that were excluded because not containing a transaction.
    pub excluded_no_transaction_blocks: Vec<BlockId>,
    /// The blocks that were excluded because conflicting with the ledger state.
    pub excluded_conflicting_blocks: Vec<(BlockId, ConflictReason)>,
    /// The blocks that were included.
    pub included_blocks: Vec<BlockId>,
    /// The number of outputs consumed within the milestone.
    pub consumed_outputs: usize,
    /// The number of outputs created within the milestone.
    pub created_outputs: usize,
    /// Whether a receipt was included in the milestone or not.
    pub receipt: bool,
}

/// An event that indicates that a block was referenced.
#[derive(Clone)]
pub struct BlockReferenced {
    /// The block identifier of the block.
    pub block_id: BlockId,
}

/// An event that indicates that an output was consumed.
#[derive(Clone)]
pub struct OutputConsumed {
    /// The identifier of the block that contains the transaction that consumes the output.
    pub block_id: BlockId,
    /// The identifier of the consumed output.
    pub output_id: OutputId,
    /// The consumed output.
    pub output: Output,
}

/// An event that indicates that an output was created.
#[derive(Clone)]
pub struct OutputCreated {
    /// The identifier of the created output.
    pub output_id: OutputId,
    /// The created output.
    pub output: CreatedOutput,
}

/// An event that indicates that a snapshot happened.
#[derive(Clone)]
pub struct SnapshottedIndex {
    /// The snapshotted index.
    pub index: MilestoneIndex,
}

/// An event that indicates that a pruning happened.
#[derive(Clone)]
pub struct PrunedIndex {
    /// The pruned index.
    pub index: MilestoneIndex,
}

/// An event that indicates that a receipt was created.
#[derive(Clone)]
pub struct ReceiptCreated(pub Receipt);

/// An event that indicates that the ledger was updated.
#[derive(Clone)]
pub struct LedgerUpdated {
    /// Index of the confirmed milestone.
    pub milestone_index: MilestoneIndex,
    /// The outputs created within the confirmed milestone.
    pub created_outputs: HashMap<OutputId, CreatedOutput>,
    /// The outputs consumed within the confirmed milestone.
    pub consumed_outputs: HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
}
