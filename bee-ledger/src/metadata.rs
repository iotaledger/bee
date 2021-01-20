// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    conflict::ConflictReason,
    model::{Output, Spent},
};

use bee_message::{milestone::MilestoneIndex, payload::transaction::OutputId, MessageId};

use std::collections::HashMap;

/// White flag metadata of a milestone confirmation.
#[derive(Default)]
pub(crate) struct WhiteFlagMetadata {
    /// Index of the confirming milestone.
    pub(crate) index: MilestoneIndex,
    /// The number of messages which were referenced by the confirming milestone.
    pub(crate) num_referenced_messages: usize,
    /// The messages which were excluded because they did not include a transaction.
    pub(crate) excluded_no_transaction_messages: Vec<MessageId>,
    /// The messages which were excluded as they were conflicting with the ledger state.
    pub(crate) excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    // The messages which mutate the ledger in the order in which they were applied.
    pub(crate) included_messages: Vec<MessageId>,
    pub(crate) spent_outputs: HashMap<OutputId, Spent>,
    pub(crate) created_outputs: HashMap<OutputId, Output>,
}

impl WhiteFlagMetadata {
    /// Creates a new white flag metadata.
    pub(crate) fn new(index: MilestoneIndex) -> WhiteFlagMetadata {
        WhiteFlagMetadata {
            index,
            ..Self::default()
        }
    }
}
