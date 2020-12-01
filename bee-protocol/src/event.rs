// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Milestone;

use bee_message::MessageId;

pub struct LatestMilestoneChanged(pub Milestone);

pub struct LatestSolidMilestoneChanged(pub Milestone);

pub struct MessageSolidified(pub MessageId);

pub struct TpsMetricsUpdated {
    pub incoming: u64,
    pub new: u64,
    pub known: u64,
    pub invalid: u64,
    pub outgoing: u64,
}
