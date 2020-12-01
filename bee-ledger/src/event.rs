// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_protocol::MilestoneIndex;

pub struct MilestoneConfirmed {
    pub index: MilestoneIndex,
    pub timestamp: u64,
    pub messages_referenced: usize,
    pub messages_excluded_no_transaction: usize,
    pub messages_excluded_conflicting: usize,
    pub messages_included: usize,
}
