// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_message::payload::indexation::PaddedIndex;
use bee_tangle::MessageRef;

/// An event that indicates that a message was processed.
#[derive(Clone)]
pub struct MessageProcessed {
    /// Message identifier of the processed message.
    pub message_id: MessageId,
    /// The processed message
    pub message: MessageRef,
}

/// An event that indicates that a message was processed.
#[derive(Clone)]
pub struct IndexationMessage {
    /// Message identifier of the processed message.
    pub message_id: MessageId,
    /// The indexation message
    pub message: MessageRef,
    /// The index of message
    pub index: PaddedIndex,
}

/// An event that indicates that a message was solidified.
#[derive(Clone)]
pub struct MessageSolidified {
    /// Message identifier of the solidified message.
    pub message_id: MessageId,
}

/// An event that indicates that the MPS metrics were updated.
#[derive(Clone)]
pub struct MpsMetricsUpdated {
    /// Number of incoming messages.
    pub incoming: u64,
    /// Number of new messages.
    pub new: u64,
    /// Number of known messages.
    pub known: u64,
    /// Number of invalid messages.
    pub invalid: u64,
    /// Number of outgoing messages.
    pub outgoing: u64,
}

/// An event that indicates that a vertex was created.
#[derive(Clone)]
pub struct VertexCreated {
    /// Message identifier of the created vertex.
    pub message_id: MessageId,
    /// Message identifiers of the parents of the created vertex.
    pub parent_message_ids: Vec<MessageId>,
    /// Whether the vertex is a solid message.
    pub is_solid: bool,
    /// Whether the vertex is a referenced message.
    pub is_referenced: bool,
    /// Whether the vertex is a conflicting message.
    pub is_conflicting: bool,
    /// Whether the vertex is a milestone message.
    pub is_milestone: bool,
    /// Whether the vertex is a tip message.
    pub is_tip: bool,
    /// Whether the vertex is a selected message.
    pub is_selected: bool,
}

/// An event that indicates that a tip was added.
#[derive(Clone)]
pub struct TipAdded {
    /// Message identifier of the added tip.
    pub message_id: MessageId,
}

/// An event that indicates that a tip was removed.
#[derive(Clone)]
pub struct TipRemoved {
    /// Message identifier of the removed tip.
    pub message_id: MessageId,
}
