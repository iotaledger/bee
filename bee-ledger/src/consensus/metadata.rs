// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{BalanceDiffs, ConsumedOutput, CreatedOutput};

use bee_ledger_types::types::ConflictReason;
use bee_message::{milestone::MilestoneIndex, output::OutputId, MessageId};

use std::collections::HashMap;

/// White flag metadata of a milestone confirmation.
#[derive(Default)]
pub struct WhiteFlagMetadata {
    /// Index of the confirming milestone.
    pub(crate) index: MilestoneIndex,
    /// The number of messages which were referenced by the confirming milestone.
    pub(crate) referenced_messages: usize,
    /// The messages which were excluded because they did not include a transaction.
    pub(crate) excluded_no_transaction_messages: Vec<MessageId>,
    /// The messages which were excluded as they were conflicting with the ledger state.
    pub(crate) excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    // The messages which mutate the ledger in the order in which they were applied.
    pub(crate) included_messages: Vec<MessageId>,
    pub(crate) created_outputs: HashMap<OutputId, CreatedOutput>,
    pub(crate) consumed_outputs: HashMap<OutputId, ConsumedOutput>,
    pub(crate) balance_diffs: BalanceDiffs,
    pub(crate) merkle_proof: Vec<u8>,
}

impl WhiteFlagMetadata {
    /// Creates a new white flag metadata.
    pub fn new(index: MilestoneIndex) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            index,
            ..Self::default()
        }
    }

    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }
}
