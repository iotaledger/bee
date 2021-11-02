// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{BalanceDiffs, ConsumedOutput, CreatedOutput};

use bee_message::{milestone::MilestoneIndex, output::OutputId, MessageId};
use bee_tangle::ConflictReason;

use std::collections::HashMap;

/// White flag metadata of a milestone confirmation.
pub struct WhiteFlagMetadata {
    /// Index of the confirmed milestone.
    pub(crate) index: MilestoneIndex,
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
    /// The balance diffs occurring within the confirmed milestone.
    pub(crate) balance_diffs: BalanceDiffs,
    /// The merkle proof of the milestone.
    pub(crate) merkle_proof: Vec<u8>,
}

impl WhiteFlagMetadata {
    /// Creates a new `WhiteFlagMetadata`.
    pub fn new(index: MilestoneIndex) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            index,
            referenced_messages: 0,
            excluded_no_transaction_messages: Vec::new(),
            excluded_conflicting_messages: Vec::new(),
            included_messages: Vec::new(),
            created_outputs: HashMap::new(),
            consumed_outputs: HashMap::new(),
            balance_diffs: BalanceDiffs::new(),
            merkle_proof: Vec::new(),
        }
    }

    /// Returns the merkle proof of a `WhiteFlagMetadata`.
    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }
}
