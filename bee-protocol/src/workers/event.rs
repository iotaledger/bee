// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{prelude::MilestoneIndex, MessageId};

#[derive(Clone)]
pub struct MessageProcessed(pub MessageId, pub Vec<u8>);

#[derive(Clone)]
pub struct MessageSolidified(pub MessageId);

#[derive(Clone)]
pub struct MessageConfirmed {
    pub message_id: MessageId,
    pub parents: Vec<MessageId>,
    pub is_solid: bool,
    pub milestone_index: MilestoneIndex,
}

#[derive(Clone)]
pub struct MpsMetricsUpdated {
    pub incoming: u64,
    pub new: u64,
    pub known: u64,
    pub invalid: u64,
    pub outgoing: u64,
}

#[derive(Clone)]
pub struct NewVertex {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub is_solid: bool,
    pub is_referenced: bool,
    pub is_conflicting: bool,
    pub is_milestone: bool,
    pub is_tip: bool,
    pub is_selected: bool,
}

#[derive(Clone)]
pub struct TipAdded(pub MessageId);

#[derive(Clone)]
pub struct TipRemoved(pub MessageId);
