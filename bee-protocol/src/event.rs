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
