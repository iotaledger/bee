// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_block::{
    output::OutputId,
    payload::milestone::{MerkleRoot, MilestoneId, MilestoneIndex},
    semantic::ConflictReason,
    BlockId,
};

use crate::types::{ConsumedOutput, CreatedOutput};

/// White flag metadata of a milestone confirmation.
pub struct WhiteFlagMetadata {
    /// Index of the confirmed milestone.
    pub(crate) milestone_index: MilestoneIndex,
    /// Timestamp of the confirmed milestone.
    pub(crate) milestone_timestamp: u32,
    /// The id of the previous milestone.
    pub(crate) previous_milestone_id: Option<MilestoneId>,
    /// Whether the previous milestone has been found in the past cone of the current milestone.
    pub(crate) found_previous_milestone: bool,
    /// The blocks which were referenced by the confirmed milestone.
    pub(crate) referenced_blocks: Vec<BlockId>,
    /// The blocks which were excluded because they did not include a transaction.
    pub(crate) excluded_no_transaction_blocks: Vec<BlockId>,
    /// The blocks which were excluded because they were conflicting with the ledger state.
    pub(crate) excluded_conflicting_blocks: Vec<(BlockId, ConflictReason)>,
    // The blocks which mutate the ledger in the order in which they were applied.
    pub(crate) included_blocks: Vec<BlockId>,
    /// The outputs created within the confirmed milestone.
    pub(crate) created_outputs: HashMap<OutputId, CreatedOutput>,
    /// The outputs consumed within the confirmed milestone.
    pub(crate) consumed_outputs: HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    /// The confirmed merkle root of the milestone.
    pub(crate) confirmed_merkle_root: MerkleRoot,
    /// The applied merkle root of the milestone.
    pub(crate) applied_merkle_root: MerkleRoot,
}

impl WhiteFlagMetadata {
    /// Creates a new [`WhiteFlagMetadata`].
    pub fn new(
        milestone_index: MilestoneIndex,
        milestone_timestamp: u32,
        previous_milestone_id: Option<MilestoneId>,
    ) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            milestone_index,
            milestone_timestamp,
            previous_milestone_id,
            found_previous_milestone: false,
            referenced_blocks: Vec::new(),
            excluded_no_transaction_blocks: Vec::new(),
            excluded_conflicting_blocks: Vec::new(),
            included_blocks: Vec::new(),
            created_outputs: HashMap::new(),
            consumed_outputs: HashMap::new(),
            confirmed_merkle_root: MerkleRoot::null(),
            applied_merkle_root: MerkleRoot::null(),
        }
    }

    /// Returns the confirmed merkle root of a [`WhiteFlagMetadata`].
    pub fn confirmed_merkle_root(&self) -> &MerkleRoot {
        &self.confirmed_merkle_root
    }

    /// Returns the applied merkle root of a [`WhiteFlagMetadata`].
    pub fn applied_merkle_root(&self) -> &MerkleRoot {
        &self.applied_merkle_root
    }
}
