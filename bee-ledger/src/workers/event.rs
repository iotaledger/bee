// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{ConsumedOutput, CreatedOutput};

use bee_ledger_types::types::ConflictReason;
use bee_message::{milestone::MilestoneIndex, MessageId};

/// An event that indicates that a milestone was confirmed.
#[derive(Clone)]
pub struct MilestoneConfirmed {
    /// The message identifier of the milestone.
    pub message_id: MessageId,
    /// The index of the milestone.
    pub index: MilestoneIndex,
    /// The timestamp of the milestone.
    pub timestamp: u64,
    /// The number of messages referenced by the milestone.
    pub referenced_messages: usize,
    /// The messages that were excluded because not containing a transaction.
    pub excluded_no_transaction_messages: Vec<MessageId>,
    /// The messages that were excluded because conflicting with the ledger state.
    pub excluded_conflicting_messages: Vec<(MessageId, ConflictReason)>,
    /// The messages that were included.
    pub included_messages: Vec<MessageId>,
    /// The number of outputs consumed within the milestone.
    pub consumed_outputs: usize,
    /// The number of outputs created within the milestone.
    pub created_outputs: usize,
    /// Whether a receipt was included in the milestone or not.
    pub receipt: bool,
}

/// An event that indicates that an output was consumed.
pub struct OutputConsumed {
    /// The consumed output.
    pub output: ConsumedOutput,
}

/// An event that indicates that an output was created.
pub struct OutputCreated {
    /// The created output.
    pub output: CreatedOutput,
}

/// An event that indicates that a snapshot happened.
pub struct SnapshottedIndex {
    /// The snapshotted index.
    pub index: MilestoneIndex,
}

/// An event that indicates that a pruning happened.
pub struct PrunedIndex {
    /// The pruned index.
    pub index: MilestoneIndex,
}
