// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Module containing the event occurring during ledger operations.

use bee_message::{
    milestone::MilestoneIndex,
    output::{Output, OutputId},
    MessageId,
};
use bee_tangle::ConflictReason;

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

/// An event that indicates that a message was referenced.
#[derive(Clone)]
pub struct MessageReferenced {
    /// The message identifier of the message.
    pub message_id: MessageId,
}

/// An event that indicates that an output was consumed.
pub struct OutputConsumed {
    /// The identifier of the message that contains the transaction that consumes the output.
    pub message_id: MessageId,
    /// The identifier of the consumed output.
    pub output_id: OutputId,
    /// The consumed output.
    pub output: Output,
}

/// An event that indicates that an output was created.
pub struct OutputCreated {
    /// The identifier of the message that contains the transaction that creates the output.
    pub message_id: MessageId,
    /// The identifier of the created output.
    pub output_id: OutputId,
    /// The created output.
    pub output: Output,
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
