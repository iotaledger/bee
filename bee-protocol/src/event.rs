// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;
use bee_tangle::milestone::{Milestone, MilestoneIndex};

#[derive(Clone)]
pub struct LatestMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone)]
pub struct LatestSolidMilestoneChanged {
    pub index: MilestoneIndex,
    pub milestone: Milestone,
}

#[derive(Clone)]
pub struct MessageProcessed(pub MessageId);

#[derive(Clone)]
pub struct MessageSolidified(pub MessageId);

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
    pub parent1_id: String,
    pub parent2_id: String,
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
