// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use bee_message::{milestone::MilestoneIndex, output::OutputId, semantic::ConflictReason, MessageId};

use crate::types::{ConsumedOutput, CreatedOutput};

/// White flag metadata of a milestone confirmation.
pub struct WhiteFlagMetadata {
    /// Index of the confirmed milestone.
    pub(crate) milestone_index: MilestoneIndex,
    /// Timestamp of the confirmed milestone.
    pub(crate) milestone_timestamp: u32,
    /// The number of messages which were referenced by the confirmed milestone.
    pub(crate) referenced_messages: usize,
    /// The messages which were excluded because they did not include a transaction.
    pub(crate) excluded_no_transaction_messages: Vec<MessageId>,
    /// The messages which were excluded because they were conflicting with the ledger state.
    pub(crate) excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    // The messages which mutate the ledger in the order in which they were applied.
    pub(crate) included_messages: Vec<MessageId>,
    /// The outputs created within the confirmed milestone.
    pub(crate) created_outputs: HashMap<OutputId, CreatedOutput>,
    /// The outputs consumed within the confirmed milestone.
    pub(crate) consumed_outputs: HashMap<OutputId, (CreatedOutput, ConsumedOutput)>,
    /// The confirmed merkle proof of the milestone.
    pub(crate) confirmed_merkle_proof: Vec<u8>,
    /// The applied merkle proof of the milestone.
    pub(crate) applied_merkle_proof: Vec<u8>,
}

impl WhiteFlagMetadata {
    /// Creates a new [`WhiteFlagMetadata`].
    pub fn new(milestone_index: MilestoneIndex, milestone_timestamp: u32) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            milestone_index,
            milestone_timestamp,
            referenced_messages: 0,
            excluded_no_transaction_messages: Vec::new(),
            excluded_conflicting_messages: Vec::new(),
            included_messages: Vec::new(),
            created_outputs: HashMap::new(),
            consumed_outputs: HashMap::new(),
            confirmed_merkle_proof: Vec::new(),
            applied_merkle_proof: Vec::new(),
        }
    }

    /// Returns the confirmed merkle proof of a [`WhiteFlagMetadata`].
    pub fn confirmed_merkle_proof(&self) -> &[u8] {
        &self.confirmed_merkle_proof
    }

    /// Returns the applied merkle proof of a [`WhiteFlagMetadata`].
    pub fn applied_merkle_proof(&self) -> &[u8] {
        &self.applied_merkle_proof
    }
}
